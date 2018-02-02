extern crate git2;
extern crate cargo;

use std::io;
use std::fs;
use std::path::{Path, PathBuf};

use git2::Repository;

fn run() -> io::Result<()> {
    for entry in fs::read_dir("..")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let mut toml_path = PathBuf::from(path.clone());
            toml_path.push("Cargo.toml");

            if toml_path.exists() {
                let mut main_path = PathBuf::from(path.clone());
                main_path.push("src/main.rs");

                if main_path.exists() {
                    let _ = check_repo(&path);
                }
            }
        }
    }

    Ok(())
}

fn check_repo(path:&Path) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("Checking: {:?}", repo.path());

    if !repo.is_path_ignored("Cargo.lock")? {
        println!("Possible rust bin");
    }

    Ok(())
}

fn main() {
    let _ = run();
}
