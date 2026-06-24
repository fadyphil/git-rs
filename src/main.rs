mod commit;
mod config;
mod object;
mod refs;
mod tree;

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

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "git-rs",
    about = "A from-scratch implementation of Git's core object storage engine in Rust."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new git-rs repository
    Init,

    /// Provide content or type and size information for repository objects
    CatFile {
        #[arg(short = 'p', long, help = "Pretty-print object contents")]
        pretty: bool,
        #[arg(short = 't', long, help = "Display the type of the object")]
        r#type: bool, // 'type' is a reserved keyword in Rust, so we escape it with r#
        #[arg(short = 's', long, help = "Display the size of the object")]
        size: bool,
        /// The object hash to read
        hash: String,
    },

    /// Create a tree object from the current directory
    WriteTree,

    /// Compute object ID and optionally create a blob from a file
    HashObject {
        #[arg(short = 'w', help = "Actually write the object into the database")]
        write: bool,
        /// The file to hash
        file: String,
    },

    /// Create a new commit object (plumbing)
    CommitTree {
        /// The tree hash to commit
        tree_hash: String,
        #[arg(short = 'm', long, help = "Commit message")]
        message: String,
    },

    /// Record changes to the repository (porcelain)
    Commit {
        #[arg(short = 'm', long, help = "Commit message")]
        message: String,
    },
}

fn main() -> anyhow::Result<()> {
    // The magic happens here! clap reads env::args(), validates everything,
    // and populates the Cli struct.
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cmd_init(),

        Commands::CatFile {
            pretty,
            r#type,
            size,
            hash,
        } => cmd_cat_file(pretty, r#type, size, &hash),

        Commands::HashObject { write, file } => cmd_hash_object(&file, write),

        Commands::WriteTree => {
            let tree_hash = cmd_write_tree(Path::new("."))?;
            println!("{}", tree_hash);
            Ok(())
        }

        // FIXED: Used the correct destructured variables, added `?` and `println!`
        Commands::CommitTree { tree_hash, message } => {
            let commit_hash = cmd_write_commit(&tree_hash, &message, None)?;
            println!("{}", commit_hash);
            Ok(())
        }

        // ADDED: The missing Commit match arm
        Commands::Commit { message } => {
            let new_commit_hash = cmd_commit(&message)?;
            update_current_ref(&new_commit_hash)?;
            println!("{}", new_commit_hash);
            Ok(())
        }
    }
}

// DELETED: expect_args and run functions are no longer needed!

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

fn cmd_cat_file(pretty: bool, show_type: bool, show_size: bool, hash: &str) -> anyhow::Result<()> {
    let (kind, content) = read_object(hash).context("Failed to read git object from disk")?;

    if show_type {
        println!("{}", kind);
    } else if show_size {
        println!("{}", content.len());
    } else if pretty {
        print!("{}", String::from_utf8_lossy(&content));
    } else {
        bail!("Please specify a flag: -p (pretty), -t (type), or -s (size)");
    }
    Ok(())
}

fn cmd_hash_object(file: &str, write: bool) -> anyhow::Result<()> {
    let content = fs::read(file)?;
    if write {
        let hash = write_object("blob", &content).context("Failed to write object")?;
        println!("{}", hash);
    } else {
        bail!("Without -w, hashing without writing is not yet implemented. Please use -w.");
    }
    Ok(())
}

fn cmd_write_tree(path: &Path) -> anyhow::Result<String> {
    let tree_hash = write_tree(path).context("Failed to write tree")?;
    Ok(tree_hash)
}

// FIXED: Removed the `flag` parameter and the `match flag` block
fn cmd_write_commit(
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
) -> anyhow::Result<String> {
    let commit_hash = write_commit_object(tree_hash, commit_message, parent_hash)
        .context("Failed to write commit to disk")?;
    Ok(commit_hash)
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
