use std::{env, fs, io, path::Path};
use std::process::Command;

use itertools::Itertools;
use log::debug;

use crate::error::*;
use crate::types::*;

const AUR_URL_BASE: &str = "https://aur.archlinux.org/";
const RAURMAN_DIR: &str = ".config/raurman";
const PKGDB_FILE: &str = "pkgdb.json";

fn get_pkgdb_path() -> Result<Box<Path>, AppError> {
  let sudo_user_var = env::var("SUDO_USER");
  let home_var = env::var("HOME");
  let dir: String;

  match (sudo_user_var, home_var) {
    (Ok(user), _) => dir = format!("/home/{user}/{RAURMAN_DIR}"),
    (_, Ok(home)) => dir = format!("{home}/{RAURMAN_DIR}"),
    (Err(_), Err(e)) => return Err(e.into()),
  }

  Ok(Path::new(&format!("{dir}/{PKGDB_FILE}")).into())
}

fn read_pkgdb() -> Result<PackageDb, AppError> {
  let path = get_pkgdb_path()?;
  let raw_str = fs::read_to_string(path)?;

  let json = serde_json::from_str::<PackageDb>(&raw_str[..])?;

  Ok(json)
}

pub fn list_packages() {
  let db = read_pkgdb().unwrap_or_else(|err| {
    panic!("raurman: Reading {PKGDB_FILE} failed: {err}")
  });
  println!("{db}");
}

pub fn handle_sync(pkgs: &Vec<Package>) {
  let pkg_str = pkgs.iter().map(|pkg| &pkg.name).join(" ");
  let _res = Command::new("pacman").args(["--sync", &pkg_str]);
  debug!("pacman -S {:?}", pkgs)

}


pub fn handle_remove(pkgs: &Vec<Package>) {
  // let pkg_str = pkgs.iter().map(|pkg| pkg.name).join(" ");
  debug!("pacman -R {:?}", pkgs)
}

pub fn handle_save(pkgs: &Vec<Package>, op: &OpType, groups: &Vec<String>) -> Result<(), AppError> {
  let mut pkgdb = match read_pkgdb() {
    Ok(db) => db,
    Err(AppError::IoError(e)) if e.kind() == io::ErrorKind::NotFound => {
      PackageDb::empty()
    },
    Err(e) => return Err(e),
  };

  if op == &OpType::Sync { 
    pkgdb.add(pkgs, groups);
  }
  if op == &OpType::Remove { 
    pkgdb.remove(pkgs, groups);
  }

  let data = serde_json::to_string(&pkgdb)?;
  let path = get_pkgdb_path()?;
  if let Some(parent) = path.parent() {
    if !parent.is_dir() {
      fs::create_dir(parent)?;
    }
  } else {
    panic!("raurman: Reading {PKGDB_FILE} failed: $HOME/.config not found.")
  }
  fs::write(path, data)?;

  Ok(())
}


