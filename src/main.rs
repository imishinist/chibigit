use chibigit::commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
#[command(propagate_version = true)]
struct ChibiGit {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(commands::Init),
    LsFiles(commands::LsFiles),
    CatFile(commands::CatFile),
    HashObject(commands::HashObject),
}

fn main() {
    let git = ChibiGit::parse();
    match &git.command {
        Commands::Init(init) => init.run(),
        Commands::LsFiles(ls_files) => ls_files.run(),
        Commands::CatFile(cat_file) => cat_file.run(),
        Commands::HashObject(hash_object) => hash_object.run(),
    }
}
