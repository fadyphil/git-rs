mod commit;
mod config;
mod object;
mod refs;
mod tree;

use std::env;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context};

use crate::commit::write_commit_object;
use crate::object::read_object;
use crate::object::write_object;
use crate::refs::read_head;
use crate::refs::read_ref;
use crate::refs::update_current_ref;
use crate::tree::write_tree;

fn main() -> anyhow::Result<()> {
    // Reads the input arguments ex: git commit path/to/file.md
    // Note: args[0] is the path to this project , it is
    // target/debug/git-rs
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Usage: git-rs <command> [<args>]");
    }

    run(&args)
}

fn expect_args(args: &[String], expected: usize, usage: &str) -> anyhow::Result<()> {
    if args.len() != expected {
        bail!("Usage : {}", usage);
    }
    Ok(())
}

fn run(args: &[String]) -> anyhow::Result<()> {
    match args[1].as_str() {
        "init" => {
            expect_args(args, 2, "Usage git-rs init")?;
            cmd_init()?;
            Ok(())
        }
        "cat-file" => {
            expect_args(args, 4, "git-rs cat-file <-p|-t|-s> <hash>")?;
            cmd_cat_file(&args[2], &args[3])?;
            Ok(())
        }
        "write-tree" => {
            expect_args(args, 2, "git-rs write-tree")?;
            let current_path = Path::new(".");
            let tree_hash = cmd_write_tree(current_path)?;
            println!("{}", tree_hash);
            Ok(())
        }
        "hash-object" => {
            expect_args(args, 4, "git-rs hash-object -w <file>")?;
            cmd_hash_object(&args[3], &args[2])?;
            Ok(())
        }
        "commit-tree" => {
            expect_args(args, 5, "git-rs commit-tree <tree-hash> -m <message>")?;
            let commit_hash = cmd_write_commit(&args[2], &args[4], None, &args[3])?;
            println!("{}", commit_hash);
            Ok(())
        }
        "commit" => {
            expect_args(args, 4, "git-rs commit -m <message>")?;
            let new_commit_hash = cmd_commit(&args[3])?;
            update_current_ref(&new_commit_hash)?;
            println!("{}", new_commit_hash);
            Ok(())
        }
        unknown => {
            bail!(
                "Unknown command: '{}'. Run with no arguments to see usage.",
                unknown
            );
        }
    }
}

fn cmd_init() -> anyhow::Result<()> {
    fs::create_dir_all(".git/objects/info")?;
    fs::create_dir_all(".git/objects/pack/")?;
    fs::create_dir_all(".git/refs/heads/")?;
    fs::create_dir_all(".git/refs/tags/")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    if !Path::new(".git/config").exists() {
        fs::write(
            ".git/config",
            "[user]\nname = \"Your Name\"\nemail = \"you@example.com\"\n",
        )?;
    }
    Ok(())
}

fn cmd_cat_file(flag: &str, hash: &str) -> anyhow::Result<()> {
    let (kind, content) = read_object(hash).context("Failed to read git object from disk")?;
    match flag {
        "-p" => {
            print!("{}", String::from_utf8_lossy(&content))
        }
        "-t" => {
            println!("{}", &kind)
        }
        "-s" => {
            println!("{}", &content.len())
        }
        unknown => {
            bail!(
                "unknown flag {} \n -p for pretty print \n -t for type\n -s for size\n Usage git-rs cat-file <flag> <hash>",
                unknown
            );
        }
    }
    Ok(())
}

fn cmd_hash_object(file: &str, flag: &str) -> anyhow::Result<()> {
    let content = fs::read(file)?;
    match flag {
        "-w" => {
            let hash = write_object("blob", &content).context("Failed to write object")?;

            println!("{}", hash);
        }
        unknown => {
            bail!(
                "unknown flag  : {}\n Usage git-rs hash-object -w <file>",
                unknown
            );
        }
    }

    Ok(())
}

fn cmd_write_tree(path: &Path) -> anyhow::Result<String> {
    let tree_hash = write_tree(path).context("Failed to write tree")?;
    Ok(tree_hash)
}

fn cmd_write_commit(
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
    flag: &str,
) -> anyhow::Result<String> {
    match flag {
        "-m" => {
            let commit_hash = write_commit_object(tree_hash, commit_message, parent_hash)
                .context("Failed to write commit to disk")?;
            Ok(commit_hash)
        }
        unkown => {
            bail!(
                "unknown flag : {}\n Usage git-rs commit <tree-hash> -m <message>",
                unkown
            );
        }
    }
}

fn cmd_commit(commit_message: &str) -> anyhow::Result<String> {
    let current_path = Path::new(".");
    let tree_hash = write_tree(current_path).context("Failed to snapshot working directory")?;
    let path = read_head().context("Failed to read HEAD pointer")?;
    let ref_content = read_ref(&path).context("Failed to read current branch reference")?;

    let commit_hash = write_commit_object(&tree_hash, commit_message, ref_content.as_deref())
        .context("Failed to write commit to disk")?;
    Ok(commit_hash)
}
