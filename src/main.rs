/*!
Tool to automate updating minor dependency versions in rust bin projects.
*/

#![deny(missing_docs)]

extern crate cargo;
extern crate git2;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use git2::{Cred, IndexAddOption, PushOptions, RemoteCallbacks, Repository};

use cargo::core::Workspace;
use cargo::ops;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::util::{CliResult, Config};

const CLI_DIVIDER: &str = "--------------------";

fn main() -> io::Result<()> {
    let cwd = env::current_dir()?;

    println!("{}", CLI_DIVIDER);
    fs::read_dir(cwd)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|de| de.path())
        .filter(|path| {
            let mut toml_path = PathBuf::from(path);
            toml_path.push("Cargo.toml");

            let mut main_path = PathBuf::from(path);
            main_path.push("src/main.rs");

            toml_path.exists() && main_path.exists()
        })
        .for_each(|path| {
            if let Err(e) = check_repo(&path) {
                println!("cuo: {}\n{}", e, CLI_DIVIDER);
            }
        });

    Ok(())
}

fn credentials_callback(
    url: &str,
    username: Option<&str>,
    allowed: git2::CredentialType,
    cfg: &git2::Config,
) -> Result<Cred, git2::Error> {
    if allowed.contains(git2::USERNAME) {
        return Err(git2::Error::from_str("Try usernames later"));
    }

    if allowed.contains(git2::SSH_KEY) {
        let mut cred_helper = git2::CredentialHelper::new(url);
        cred_helper.config(&cfg);

        let name = username
            .map(|s| s.to_string())
            .or_else(|| cred_helper.username.clone())
            .or_else(|| std::env::var("USER").ok())
            .or_else(|| std::env::var("USERNAME").ok())
            .or_else(|| Some("git".to_string()))
            .unwrap();

        // TODO better handling of ssh key than just grabbing default, prob using ssh-agent
        let mut pk_path = std::env::home_dir().unwrap();
        pk_path.push(".ssh/id_rsa");
        let result = Cred::ssh_key(&name, None, &pk_path, None);

        if result.is_ok() {
            return result;
        }
    }

    if allowed.contains(git2::USER_PASS_PLAINTEXT) {
        if let Ok(token) = std::env::var("GH_TOKEN") {
            return Cred::userpass_plaintext(&token, "");
        } else if let Ok(cred_helper) = Cred::credential_helper(&cfg, url, username) {
            return Ok(cred_helper);
        }
    }

    if allowed.contains(git2::DEFAULT) {
        return Cred::default();
    }

    Err(git2::Error::from_str("cuo: no authentication available"))
}

fn check_repo(path: &Path) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("cuo: Checking: {:?}", repo.path());

    if repo.statuses(None)?
        .iter()
        .filter(|s| s.status() != git2::STATUS_IGNORED)
        .count() != 0
    {
        return Err(git2::Error::from_str("Repo not clean"));
    }

    if !repo.is_path_ignored("Cargo.lock")? {
        println!("cuo: Updating rust bin project");
        let _ = cargo_update(path);

        if repo.statuses(None)?
            .iter()
            .any(|s| s.status() == git2::STATUS_WT_MODIFIED)
        {
            println!("cuo: Deps updated");
            let mut index = repo.index()?;
            let current_head = repo.head()?.peel_to_commit()?;
            let sig = repo.signature()?;

            let mut index_add_ops = IndexAddOption::empty();
            index_add_ops.insert(git2::ADD_DEFAULT);
            index_add_ops.insert(git2::ADD_CHECK_PATHSPEC);
            index.add_all(Vec::<String>::new(), index_add_ops, None)?;

            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            index.write()?;

            // TODO better commit message describing what was updated
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Update deps",
                &tree,
                &[&current_head],
            )?;

            let cfg = repo.config()?;
            let mut rcbs = RemoteCallbacks::new();
            rcbs.credentials(|u, un, a| credentials_callback(u, un, a, &cfg));
            let mut push_ops = PushOptions::default();
            push_ops.remote_callbacks(rcbs);

            // TODO don't just push master refspec
            repo.find_remote("origin")?
                .push(&["refs/heads/master"], Some(&mut push_ops))?;
        }

        println!("cuo: Done!\n{}", CLI_DIVIDER);
    }

    Ok(())
}

fn cargo_update(path: &Path) -> CliResult {
    let config = Config::default()?;
    let root = find_root_manifest_for_wd(None, path)?;

    let update_opts = ops::UpdateOptions {
        aggressive: false,
        precise: None,
        to_update: &Vec::new(),
        config: &config,
    };

    let ws = Workspace::new(&root, &config)?;
    ops::update_lockfile(&ws, &update_opts)?;
    Ok(())
}
