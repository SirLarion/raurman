use clap::Parser;

//
// Simple tool to combine pacman and AUR package management on Arch Linux 
// systems. Optionally logs which packages have been downloaded into a JSON file 
// for easy system reproducibility
//

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // The package to operate on. Corresponds directly to the name of the 
    // package with pacman or to the name of the repo in the AUR (without .git)
    pkg: String,

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
}

fn main() {
    let Args { pkg, sync, remove, aur, save } = Args::parse();
    println!("{}, sync: {}, remove: {}, aur: {}, save: {}", pkg, sync, remove, aur, save);
}
