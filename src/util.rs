use std::path::{Path, PathBuf};
use std::{env, fs, io, rc::Rc, collections::HashMap};
use std::process::{Command, Stdio};

use itertools::Itertools;
use log::debug;

use crate::error::*;
use crate::types::*;

const AUR_URL_BASE: &str = "https://aur.archlinux.org";
const AUR_TMP_DIR: &str = "/tmp/pkgdir";
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

pub fn backup_pkgdb(to: &String) -> Result<(), AppError> {
  let from = get_pkgdb_path()?;
  let mut to = PathBuf::from(to);

  if to.is_dir() {
    to = to.join(PKGDB_FILE);
  }
  fs::copy(from, to)?; 

  Ok(())
}

pub fn list_packages(groups: Vec<Rc<str>>) {
  let db = read_pkgdb().unwrap_or_else(|err| {
    panic!("raurman: Reading {PKGDB_FILE} failed: {err}")
  });

  if !groups.is_empty() {
    for g in groups {
      if let Some(list) = db.json.get(&g) {
        let print_group = PackageDb { json: HashMap::from([(g, list.clone())]) };
        println!("{print_group}");
      }
    }
  } else {
    println!("{db}");
  }
}

fn install_aur_pkg(pkg: &Package) -> Result<(), AppError> {
  let name = &pkg.name;

  // Remove possible previous temp dir and contents
  if let Err(_) = fs::remove_dir_all(AUR_TMP_DIR) {};

  // Clone AUR package and cd into it
  debug!("git clone {AUR_URL_BASE}/{name}.git {AUR_TMP_DIR}");
  Command::new("git")
    .args(["clone", 
      format!("{AUR_URL_BASE}/{name}.git").as_str(), 
      format!("{AUR_TMP_DIR}").as_str()
    ])
    .status()?;

  debug!("cd {AUR_TMP_DIR}");
  env::set_current_dir(AUR_TMP_DIR)?;

  let res = Command::new("makepkg")
    .arg("-si")
    .stdout(Stdio::inherit())
    .status();

  let exit_status = res?;

  if exit_status.success() {
    Ok(())
  } else {
    Err(AppError::CmdError(exit_status))
  }
}

fn install_pacman_pkgs(pkgs: Vec<&Package>) -> Result<(), AppError> {
  let str_pkgs: Vec<String> = pkgs
    .iter()
    .map(|pkg| format!("{}", &pkg.name))
    .collect();

  debug!("pacman -S {}", str_pkgs.join(" "));
  Command::new("pacman")
    .arg("--sync")
    .args(str_pkgs)
    .stdout(Stdio::inherit())
    .status()?;

  Ok(())
}

pub fn use_db_pkgs_if_empty
  (pkgs: Vec<Package>, groups: &Vec<Rc<str>>) -> Result<(Vec<Package>, bool), AppError> {
  if !pkgs.is_empty() {
    return Ok((pkgs, false));
  }

  let pkgdb = read_pkgdb()?;
  let mut pkgs: Vec<Package> = Vec::new();

  for (g, mut list) in pkgdb.json.into_iter() {
    if groups.is_empty() || groups.contains(&g) {
      pkgs.append(&mut list);
    }
  }

  Ok((pkgs, true))
}

pub fn handle_sync(pkgs: &Vec<Package>) -> Result<(), AppError> {
  for (aur, pkgs) in &pkgs.into_iter().group_by(|pkg| pkg.aur.is_some()) {
    // AUR packages
    if aur {
      for pkg in pkgs.into_iter() {
        install_aur_pkg(pkg)?;
      }
    }
    // Pacman packages
    else {
      install_pacman_pkgs(pkgs.collect())?;
    }
  }

  Ok(())
}


pub fn handle_remove(pkgs: &Vec<Package>) -> Result<(), AppError> {
  let pkgs_str = pkgs.iter().map(|pkg| &pkg.name).join(" ");
  debug!("pacman -R {:?}", pkgs);
  Command::new("pacman")
    .args(["--remove", &pkgs_str])
    .stdout(Stdio::inherit())
    .status()?;

  Ok(())
}

pub fn handle_save(pkgs: Vec<Package>, op: &OpType, groups: Vec<Rc<str>>) -> Result<(), AppError> {
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
  else if op == &OpType::Remove { 
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


