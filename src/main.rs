mod object;

use std::env;
use std::fs;

use crate::object::read_object;
use crate::object::write_object;

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

fn run(args: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    match args[1].as_str() {
        "init" => {
            if args.len() != 2 {
                eprintln!("Usage git-rs init");
                std::process::exit(1);
            }
            create_dir_structure()?;
            Ok(())
        }
        "cat-file" => {
            if args.len() < 3 {
                eprintln!("Usage: git-rs cat-file -p <file>");
                std::process::exit(1)
            }
            let content = read_object(&args[2]);
            println!("{:?}", content);
            Ok(())
        }
        "write-tree" => Ok(()),
        "hash" => {
            if args.len() < 3 {
                eprintln!("Usage git-rs hash <file>");
                std::process::exit(1)
            }
            let file = read_file(&args[2])?;

            let created_obj = write_object("blob", file.as_bytes());
            println!(
                "This is the result of write_object func :  {:?}",
                created_obj
            );
            Ok(())
        }
        unknown => {
            eprintln!("Unknown Command : {}", unknown);
            std::process::exit(1);
        }
    }
}

fn read_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}

fn create_dir_structure() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(".git/objects/info")?;
    fs::create_dir_all(".git/objects/pack/")?;
    fs::create_dir_all(".git/refs/heads/")?;
    fs::create_dir_all(".git/refs/tags/")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    Ok(())
}
