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
    let repodir =
        std::env::current_dir().context("Failed to determine the current working directory")?;

    match cli.command {
        Commands::Init => cmd_init(&repodir),

        Commands::CatFile {
            pretty,
            r#type,
            size,
            hash,
        } => cmd_cat_file(pretty, r#type, size, &hash, &repodir),

        Commands::HashObject { write, file } => cmd_hash_object(&file, write, &repodir),

        Commands::WriteTree => {
            let tree_hash = cmd_write_tree(Path::new("."), &repodir)?;
            println!("{}", tree_hash);
            Ok(())
        }

        // FIXED: Used the correct destructured variables, added `?` and `println!`
        Commands::CommitTree { tree_hash, message } => {
            let commit_hash = cmd_write_commit(&tree_hash, &message, None, &repodir)?;
            println!("{}", commit_hash);
            Ok(())
        }

        // ADDED: The missing Commit match arm
        Commands::Commit { message } => {
            let new_commit_hash = cmd_commit(&message, &repodir)?;
            update_current_ref(&new_commit_hash, &repodir)?;
            println!("{}", new_commit_hash);
            Ok(())
        }
    }
}

// DELETED: expect_args and run functions are no longer needed!

fn cmd_init(repo_dir: &Path) -> anyhow::Result<()> {
    let git_dir = repo_dir.join(".git");
    fs::create_dir_all(git_dir.join("objects/info"))?;
    fs::create_dir_all(git_dir.join("objects/pack/"))?;
    fs::create_dir_all(git_dir.join("refs/heads/"))?;
    fs::create_dir_all(git_dir.join("refs/tags/"))?;
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

    let config_path = git_dir.join("config");
    if !config_path.exists() {
        fs::write(
            config_path,
            "[user]\nname = \"Your Name\"\nemail = \"you@example.com\"\n",
        )?;
    }
    Ok(())
}

fn cmd_cat_file(
    pretty: bool,
    show_type: bool,
    show_size: bool,
    hash: &str,
    dir: &Path,
) -> anyhow::Result<()> {
    let (kind, content) = read_object(hash, dir).context("Failed to read git object from disk")?;

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

fn cmd_hash_object(file: &str, write: bool, dir: &Path) -> anyhow::Result<()> {
    let content = fs::read(file)?;
    if write {
        let hash = write_object("blob", &content, dir).context("Failed to write object")?;
        println!("{}", hash);
    } else {
        bail!("Without -w, hashing without writing is not yet implemented. Please use -w.");
    }
    Ok(())
}

fn cmd_write_tree(path: &Path, dir: &Path) -> anyhow::Result<String> {
    let tree_hash = write_tree(path, dir).context("Failed to write tree")?;
    Ok(tree_hash)
}

// FIXED: Removed the `flag` parameter and the `match flag` block
fn cmd_write_commit(
    tree_hash: &str,
    commit_message: &str,
    parent_hash: Option<&str>,
    dir: &Path,
) -> anyhow::Result<String> {
    let commit_hash = write_commit_object(tree_hash, commit_message, parent_hash, dir)
        .context("Failed to write commit to disk")?;
    Ok(commit_hash)
}

fn cmd_commit(commit_message: &str, dir: &Path) -> anyhow::Result<String> {
    let current_path = Path::new(".");
    let tree_hash =
        write_tree(current_path, dir).context("Failed to snapshot working directory")?;
    let path = read_head(dir).context("Failed to read HEAD pointer")?;
    let ref_content = read_ref(&path, dir).context("Failed to read current branch reference")?;

    let commit_hash = write_commit_object(&tree_hash, commit_message, ref_content.as_deref(), dir)
        .context("Failed to write commit to disk")?;
    Ok(commit_hash)
}
