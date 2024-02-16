use std::collections::HashMap;
use std::{fmt, cmp::Ordering, rc::Rc};

use clap::{Parser, Args};
use serde::{Serialize, Deserialize, Serializer, ser::SerializeStruct};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct Cli {
  /// The package(s) to operate on. Corresponds directly to the name of the 
  /// package with pacman or to the name of the repo in the AUR (without .git)
  pub pkgs: Vec<String>,

  #[command(flatten)]
  pub op: Operation,

  /// Whether to look for the package in the AUR rather than the pacman database
  #[arg(short = 'A', long, default_value_t = false)]
  pub aur: bool,

  #[command(flatten)]
  pub db: DbOpts,

  /// Run command verbosely
  #[arg(long, default_value_t = false)]
  pub verbose: bool,

  /// Turn debugging information on
  #[arg(short, long, default_value_t = false)]
  pub debug: bool,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Operation {
  /// The equivalent of "-S" in pacman
  #[arg(short = 'S', long, default_value_t = false)]
  pub sync: bool,

  /// The equivalent of "-R" in pacman
  #[arg(short = 'R', long, default_value_t = false)]
  pub remove: bool, 

  /// Print out the packages that have been installed via raurman
  #[arg(short = 'L', long, default_value_t = false)]
  pub list: bool,

  /// Create a backup of the pkgdb.json in the specified location
  #[arg(long = "backup")]
  pub backup: Option<String>,
}

#[derive(Args)]
#[group()]
pub struct DbOpts {
  /// Whether to save the effect of the operation in pkgdb.json  
  #[arg(short, long, help_heading = "Database options", default_value_t = false)]
  pub save: bool,

  /// Which groups to save the package under, comma separated list
  #[arg(
    short = 'G', long, help_heading = "Database options", 
    value_delimiter = ',', default_values_t = Vec::<String>::new()
  )]
  pub groups: Vec<String>,

  /// Only perform the selected operation on the database
  #[arg(long = "db-only", help_heading = "Database options", requires = "save", default_value_t = false)]
  pub db_only: bool,
}

#[derive(PartialEq)]
pub enum OpType {
  Sync,
  Remove,
  List,
  Backup(String),
}

impl From<Operation> for OpType {
  fn from(op: Operation) -> OpType {
    match op {
      Operation { sync: true, .. }       => OpType::Sync,
      Operation { remove: true, .. }     => OpType::Remove,
      Operation { list: true, .. }       => OpType::List,
      Operation { backup: Some(to), .. } => OpType::Backup(to),
      _ => panic!("Something has gone terribly wrong 👾")
    }
  }
}

//
// Object representation of a single package in pkgdb.json
//
#[derive(Deserialize, Debug, Clone, Eq)]
pub struct Package { 
  pub name: Rc<str>,
  pub aur: Option<bool> 
}

impl PartialEq for Package {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

impl Package {
  pub fn new(name: &str, aur: bool) -> Package {
    Package { name: name.into(), aur: if aur { Some(true) } else { None } }
  }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl Serialize for Package {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("Package", 2)?;
    state.serialize_field("name", &self.name)?;

    if let Some(_) = self.aur {
      state.serialize_field("aur", &self.aur)?;
    } else {
      state.skip_field("aur")?;
    }

    state.end()

  }
}

//
// Object representation of the contents of pkgdb.json
//
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageDb {
  #[serde(flatten)]
  pub json: HashMap<Rc<str>, Vec<Package>>
}

impl PackageDb {
  // Initialize empty PackageDb
  pub fn empty() -> PackageDb {
    PackageDb { json: HashMap::from([("default".into(), Vec::<Package>::new())]) }
  }
  
  // Add package(s) to pkgdb, if no group is defined, apply to all groups
  pub fn add(&mut self, pkgs: Vec<Package>, groups: Vec<Rc<str>>) -> &PackageDb {
    let mut new_groups: Vec<(Rc<str>, Vec<Package>)> = Vec::new();

    if !groups.is_empty() {
      for g in groups {
        match self.json.get_mut(&g) {
          Some(list) => {
            list.append(&mut pkgs.clone()); 
            list.sort(); 
            list.dedup(); 
          },
          None => new_groups.push((g, pkgs.clone()))
        }
      }
      for (g, list) in new_groups {
        self.json.insert(g, list);
      }
    } else {
      for (_, list) in self.json.iter_mut() {
        list.append(&mut pkgs.clone()); 
        list.sort(); 
        list.dedup();        
      }
    }

    self
  } 

  // Remove package(s) from pkgdb, if no group is defined, apply to all groups
  pub fn remove(&mut self, pkgs: Vec<Package>, groups: Vec<Rc<str>>) -> &PackageDb {
    for (g, list) in self.json.clone().into_iter() {
      if groups.is_empty() || groups.contains(&g) {
        let filtered: Vec<Package> = list.into_iter()
          .filter(|pkg| !pkgs.contains(pkg))
          .collect();
        
        if filtered.is_empty() {
          self.json.remove(&g);
        } else {
          self.json.insert(g, filtered);
        }
      }
    }

    self
  } 
}

impl fmt::Display for PackageDb {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let json = &self.json;
    let mut out = String::new();

    for (category, pkgs) in json.into_iter() {
      out.push_str(&format!("{}: \n", category));

      for Package { name, aur } in pkgs {
        out.push_str(&format!("|  {}", name));
        if let Some(true) = aur {
          out.push_str(", AUR");
        } 

        out.push_str("\n");
      }
      out.push_str("\n");
    }

    write!(f, "{}", out)
  }
}
