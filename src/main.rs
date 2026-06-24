mod commit;
mod config;
mod object;
mod refs;
mod tree;

use std::env;
use std::fs;
use std::path::Path;

use crate::commit::write_commit_object;
use crate::object::read_object;
use crate::object::write_object;
use crate::refs::read_head;
use crate::refs::read_ref;
use crate::refs::update_current_ref;
use crate::tree::write_tree;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads the input arguments ex: git commit path/to/file.md
    // Note: args[0] is the path to this project , it is
    // target/debug/git-rs
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: git-rs <command> [<args>]");
        std::process::exit(1);
    }

    run(&args)
}

fn expect_args(args: &[String], expected: usize, usage: &str) {
    if args.len() != expected {
        eprintln!("Usage : {}", usage);
        std::process::exit(1)
    }
}

fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args[1].as_str() {
        "init" => {
            expect_args(args, 2, "Usage git-rs init");
            cmd_init()?;
            Ok(())
        }
        "cat-file" => {
            expect_args(args, 4, "git-rs cat-file <-p|-t|-s> <hash>");
            cmd_cat_file(&args[2], &args[3])?;
            Ok(())
        }
        "write-tree" => {
            expect_args(args, 2, "git-rs write-tree");
            let current_path = Path::new(".");
            let tree_hash = cmd_write_tree(current_path)?;
            println!("{}", tree_hash);
            Ok(())
        }
        "hash-object" => {
            expect_args(args, 4, "git-rs hash-object -w <file>");
            cmd_hash_object(&args[3], &args[2])?;
            Ok(())
        }
        "commit-tree" => {
            expect_args(args, 5, "git-rs commit-tree <tree-hash> -m <message>");
            let commit_hash = cmd_write_commit(&args[2], &args[4], None, &args[3])?;
            println!("{}", commit_hash);
            Ok(())
        }
        "commit" => {
            expect_args(args, 4, "git-rs commit -m <message>");
            let new_commit_hash = cmd_commit(&args[3])?;
            update_current_ref(&new_commit_hash)?;
            println!("{}", new_commit_hash);
            Ok(())
        }
        unknown => {
            eprintln!("Unknown Command : {}", unknown);
            std::process::exit(1);
        }
    }
}

fn cmd_init() -> Result<(), Box<dyn std::error::Error>> {
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

fn cmd_cat_file(flag: &str, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (kind, content) = read_object(hash)?;
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
            eprintln!(
                "unknown flag {} \n -p for pretty print \n -t for type\n -s for size\n Usage git-rs cat-file <flag> <hash>",
                unknown
            );
            std::process::exit(1)
        }
    }
    Ok(())
}

fn cmd_hash_object(file: &str, flag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read(file)?;
    match flag {
        "-w" => {
            let hash = write_object("blob", &content)?;

            println!("{}", hash);
        }
        unknown => {
            eprintln!(
                "unknown flag  : {}\n Usage git-rs hash-object -w <file>",
                unknown
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_write_tree(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let tree_hash = write_tree(path)?;
    Ok(tree_hash)
}

fn cmd_write_commit(
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match flag {
        "-m" => {
            let commit_hash = write_commit_object(tree_hash, commit_message, parent_hash)?;
            Ok(commit_hash)
        }
        unkown => {
            eprintln!(
                "unknown flag : {}\n Usage git-rs commit <tree-hash> -m <message>",
                unkown
            );
            std::process::exit(1);
        }
    }
}

fn cmd_commit(commit_message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let current_path = Path::new(".");
    let tree_hash = write_tree(current_path)?;
    let path = read_head()?;
    let ref_content = read_ref(&path)?;

    let commit_hash = write_commit_object(&tree_hash, commit_message, ref_content.as_deref())?;
    Ok(commit_hash)
}
