mod object;
mod tree;

use std::env;
use std::fs;
use std::path::Path;

use crate::object::read_object;
use crate::object::write_object;
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

fn run(args: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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
            let tree_hash = cmd_write_tree(&current_path)?;
            println!("{}", tree_hash);
            Ok(())
        }
        "hash-object" => {
            expect_args(args, 4, "git-rs hash-object -w <file>");
            cmd_hash_object(&args[3], &args[2])?;
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
