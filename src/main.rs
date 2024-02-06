use clap::Parser;
use std::path;

//
// Simple tool to combine pacman and AUR package management on Arch Linux 
// systems. Optionally logs which packages have been downloaded into a JSON file.
// The file location can be specified with --file (-f) and defaults to 
// $XDG_CONFIG_HOME/raurman/packagedb.json
//

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Sync, the equivalent of "-S" in pacman
    #[arg(short = 'S', long, default_value_t = false)]
    sync: bool,

    // Remove, the equivalent of "-R" in pacman
    #[arg(short = 'R', long, default_value_t = false)]
    remove: bool,

    // Whether to look for the package in the AUR rather than pacman
    #[arg(short, long, default_value_t = false)]
    aur: bool,
    
    // Whether to save the installed package 
    #[arg(short, long, default_value_t = false)]
    save: bool,

    #[arg(short, long)]
    file: path::PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("args: {:?}", args);
}
