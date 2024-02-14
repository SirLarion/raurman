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

  let op: OpType = op.into();

  let _ = logger::init(LoggerFlags { verbose, debug });

  let pkg_objs = pkgs.iter().map(|pkg| { Package::new(pkg.as_str(), aur)}).collect();

  // Return early if listing or creating backup
  if op == List {
    list_packages();
    return Ok(());
  }
  if let Backup(to) = op {
    return backup_pkgdb(&to);
  }

  let has_correct_rights = 
    !Uid::effective().is_root() && 
    !debug && 
    !db.db_only &&
    !aur;

  // Check for sudo rights
  if has_correct_rights {
    panic!("raurman: You cannot perform this operation unless you are root.")
  }

  if !db.db_only {
    if op == Sync  {
      handle_sync(&pkg_objs)?;
    } 

    // pacman can be used to remove AUR packages as well
    if op == Remove {
      handle_remove(&pkg_objs)?;
    }
  }

  if db.save {
    if let Err(e) = handle_save(&pkg_objs, &op, &db.group) {
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
