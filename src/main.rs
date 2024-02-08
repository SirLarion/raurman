use clap::{Parser, Args};
use nix::unistd::Uid;
use serde_json::{Map};

use std::error::Error;
use std::process::Command;
use std::{fs, fmt, env};

//
// Simple tool to combine pacman and AUR package management on Arch Linux 
// systems. Optionally logs which packages have been downloaded into a JSON file 
// for easy system reproducibility
//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  /// The package to operate on. Corresponds directly to the name of the 
  /// package with pacman or to the name of the repo in the AUR (without .git)
  pkg: String,

  #[command(flatten)]
  op: Operation,

  /// Whether to look for the package in the AUR rather than pacman
  #[arg(short, long, default_value_t = false)]
  aur: bool,
  
  /// Whether to save the installed package 
  #[arg(short, long, default_value_t = false)]
  save: bool,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Operation {
  /// The equivalent of "-S" in pacman
  #[arg(short = 'S', long, default_value_t = false)]
  sync: bool,

  /// The equivalent of "-R" in pacman
  #[arg(short = 'R', long, default_value_t = false)]
  remove: bool, 

  /// Print out the packages that have been installed via raurman
  #[arg(short = 'L', long, default_value_t = false)]
  list: bool,
}

#[derive(Deserialize, Debug)]
struct Package { 
  name: String,
  aur: bool 
}

type PackageDb = Box<Map<String, Vec<Package>>>;


const AUR_URL_BASE: &'static str = "https://aur.archlinux.org/";
const RAURMAN_DIR: &'static str = ".config/raurman";

fn read_pkgdb() -> Result<PackageDb, Box<dyn std::error::Error>> {
  let sudo_user_var = env::var("SUDO_USER");
  let home_var = env::var("HOME");
  let mut dir = String::new();

  match (sudo_user_var, home_var) {
    (Ok(user), _) => dir = format!("/home/{}/{}", user, RAURMAN_DIR),
    (_, Ok(home)) => dir = format!("{}/{}", home, RAURMAN_DIR),
    (Err(e), _)        => return Err(Box::new(e)),
    (_, Err(e))        => return Err(Box::new(e)),
  }

  let Ok(raw_str) = String::from_utf8(fs::read(format!("{}/pkgdb.json", dir))?) else {
    panic!("Bollocks")
  };

  let Ok(json) = serde_json::from_str::<PackageDb>(&raw_str[..]) else {
    panic!("Bollocks")
  };

  return Ok(PackageDb);
}

fn list_packages() {
  println!("{:?}", read_pkgdb());
}

fn handle_pacman_sync(pkg: &String) {
  println!("{}", pkg);
}

fn handle_aur_sync(pkg: &String) {
  println!("{}{}.git", AUR_URL_BASE, pkg);
}

fn main() {
    let Cli { pkg, op, aur, save } = Cli::parse();
    let Operation { sync, remove, list } = op; 

    // Check for sudo rights
    if (op.sync || op.remove) && !Uid::effective().is_root() {
      panic!("raurman: You cannot perform this operation unless you are root.")
    }

    if list {
      list_packages();
      return;
    }

    if sync  {
      if aur  {
        handle_aur_sync(&pkg);
      } else {
        handle_pacman_sync(&pkg);
      }
    } 

    if remove {
      println!("remove")
    }

    println!("save: {}", save);
}
