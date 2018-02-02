extern crate git2;

use std::io;
use std::io::{Error, ErrorKind};
use std::fs;
use std::path::Path;

use git2::Repository;

fn run() -> io::Result<()> {
    for entry in fs::read_dir("..")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {    
            check_repo(&path)?
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

    Ok(())
}

fn main() {
    let _ = run();
}
