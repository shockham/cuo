extern crate git2;

use std::io;
use std::io::{Error, ErrorKind};
use std::fs;
use std::path::{Path, PathBuf};

use git2::Repository;

fn run() -> io::Result<()> {
    for entry in fs::read_dir("..")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {    
            let mut toml_path = PathBuf::new();
            toml_path.push(path.clone());
            toml_path.push("Cargo.toml");

            if toml_path.exists() {
                let mut main_path = PathBuf::new();
                main_path.push(path.clone());
                main_path.push("src");
                main_path.push("main.rs");

                if main_path.exists() {
                    let _ = check_repo(&path);
                }
            }
        }
    }
    
    Ok(())
}

fn check_repo(path:&Path) -> io::Result<()> {
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(e) => return Err(Error::new(ErrorKind::Other, e.message())),
    };

    println!("Checking: {:?}", repo.path());

    if !repo.is_path_ignored("Cargo.lock").unwrap() {
        println!("Possible rust bin");
    }

    Ok(())
}

fn main() {
    let _ = run();
}
