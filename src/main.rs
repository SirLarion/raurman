use std::{io::Read, rc::Rc};

use clap::Parser;
use error::AppError;
use nix::unistd::Uid;
use log::error;

mod util;
mod logger;
pub mod types;
pub mod error;

use util::*;
use types::*;
use logger::LoggerFlags;

//
// Simple tool to combine pacman and AUR package management on Arch Linux 
// systems. Optionally logs which packages have been downloaded into a JSON file 
// for easy system reproducibility
//
fn main() -> Result<(), AppError> {
  use OpType::*;
  let Cli { pkgs, op, aur, db, verbose, debug } = Cli::parse();
  let _ = logger::init(LoggerFlags { verbose, debug });

  // Preprocess input
  let op: OpType = op.into();
  let groups: Vec<Rc<str>> = db.groups.into_iter().map(|g| g.into()).collect();

  let mut pkgs = pkgs;
  pkgs.sort();
  pkgs.dedup();
  let pkg_objs = pkgs.iter().map(|pkg| { Package::new(pkg.as_str(), aur)}).collect();

  // Return early if listing or creating backup
  if op == List {
    list_packages(groups);
    return Ok(());
  }
  if let Backup(to) = op {
    return backup_pkgdb(&to);
  }

  // Check for sudo rights where necessary
  let has_correct_rights = 
    !Uid::effective().is_root() && 
    !debug && 
    !db.db_only;

  if has_correct_rights {
    panic!("raurman: You cannot perform this operation unless you are root.")
  }

  let (pkg_objs, is_target_from_db) = use_db_pkgs_if_empty(pkg_objs, &groups)?;

  if !db.db_only {
    let handler: fn(&Vec<Package>) -> Result<(), AppError>;
    let op_string: &str;

    if op == Sync  {
      handler = handle_sync;
      op_string = "install";
    } 
    // pacman can be used to remove AUR packages as well
    else {
      handler = handle_remove;
      op_string = "remove";
    }

    if is_target_from_db {
      println!("This will {op_string} many packages. Do you want to continue? [y/N]");

      let input: Option<char> = std::io::stdin()
        .bytes() 
        .next()
        .and_then(|result| result.ok())
        .map(|byte| byte as char);

      if input != Some('y') && input != Some('Y') {
        return Ok(());
      }
    }
    handler(&pkg_objs)?;
  }

  if db.save {
    if let Err(e) = handle_save(pkg_objs, &op, groups) {
      error!("Error saving pkgdb.json: {e}");
      error!("Please resolve your pkgdb issue and rerun this command with --db-only: ");

      let op_flag = match op {
        Sync => "-S",
        Remove => "-R", 
        _ => ""
      };
      
      let pkg_str = pkgs.join(" ");

      error!("raurman {op_flag} {pkg_str} --db-only");
    };
  }
  
  Ok(())
}
