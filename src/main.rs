use std::{io::Read, rc::Rc};

use clap::Parser;
use error::AppError;
use log::error;
use nix::unistd::Uid;

pub mod error;
mod logger;
pub mod types;
mod util;

use logger::LoggerFlags;
use types::*;
use util::*;

//
// Simple tool to combine pacman and AUR package management on Arch Linux
// systems. Optionally logs which packages have been downloaded into a JSON file
// for easy system reproducibility
//
fn main() -> Result<(), AppError> {
    use OpType::*;
    let Cli {
        pkgs,
        op,
        aur,
        db,
        verbose,
        debug,
    } = Cli::parse();
    let _ = logger::init(LoggerFlags { verbose, debug });

    // Preprocess input
    let op: OpType = op.into();

    let target_db = db.groups.is_some();
    let groups: Vec<Rc<str>> = if target_db {
        db.groups.unwrap().into_iter().map(|g| g.into()).collect()
    } else {
        vec![]
    };

    let mut pkgs = pkgs;
    pkgs.sort();
    pkgs.dedup();

    let no_defined_target = pkgs.is_empty();

    let pkg_objs: Vec<Package> = if no_defined_target {
        use_db_pkgs(&groups)?
    } else {
        pkgs.clone()
            .into_iter()
            .map(|pkg| Package::new(pkg.as_str(), aur))
            .collect()
    };

    // Return early with "dry run" ops
    match op {
        Search => {
            if no_defined_target {
                return Err(AppError::ParamError("No target defined".to_string()));
            }
            return handle_search(&pkg_objs);
        }
        List => return list_packages(groups),
        Backup(to) => return backup_pkgdb(&to),
        _ => {}
    }

    let has_sudo_rights = Uid::effective().is_root();

    if !debug && !db.db_only {
        match (aur, has_sudo_rights, &op) {
            (true, true, Sync) => return Err(
                AppError::AclError(
                    "Running makepkg as root is not allowed as it can cause permanent, catastrophic damage to your system.".into()
                )
            ),
            (false, false, _) => return Err(
                AppError::AclError(
                    "You cannot perform this operation unless you are root.".into()
                )
            ),
            _ => {}
        }
    }

    let mut pkg_handler_res: Result<(), AppError> = Ok(());

    if !db.db_only {
        let handler: fn(&Vec<Package>) -> Result<(), AppError>;
        let op_string: &str;

        if op == Sync {
            handler = handle_sync;
            op_string = "install";
        }
        // pacman can be used to remove AUR packages as well
        else {
            handler = handle_remove;
            op_string = "remove";
        }

        // Use DB packages as target if none defined
        if no_defined_target {
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
        pkg_handler_res = handler(&pkg_objs);
    }

    if target_db && pkg_handler_res.is_ok() {
        if let Err(e) = handle_save(pkg_objs, &op, groups) {
            error!("Error saving pkgdb.json: {e}");
            error!("Please resolve your pkgdb issue and rerun this command with --db-only: ");

            let op_flag = match op {
                Sync => "-S",
                Remove => "-R",
                _ => "",
            };

            let pkg_str = pkgs.join(" ");

            error!("raurman {op_flag} {pkg_str} --db-only");
        };
    }

    Ok(pkg_handler_res?)
}
