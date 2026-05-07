# git-rs

🚧 **Status:** Phase 2 Complete | Phase 3 In Progress  
📖 **Blueprint:** [Build Git From Scratch in Rust](docs/build-git-in-rust.md)

## 🌟 Overview

`git-rs` is a minimal, from-scratch implementation of Git's core object storage engine, written in Rust. This project is not intended to replace Git, but to demystify it. By building Git's content-addressable storage, SHA-1 hashing, and Zlib compression pipelines from first principles, you'll understand how version control actually works at the byte level.

## ✅ Implemented Features

- `init` - Creates the `.git/` directory skeleton and `HEAD` pointer
- `hash-object -w <file>` - Reads a file, constructs the Git blob format, computes SHA-1, compresses with Zlib, and stores it in `.git/objects/`
- `cat-file <-p|-t|-s> <hash>` - Locates, decompresses, parses, and displays stored objects
- 🔄 `write-tree` & `ls-tree` - *(In Progress)* Directory snapshot engine with binary tree serialization

## 🛠️ Architecture & Design

- **Content-Addressable Storage:** Every object is stored as `.git/objects/XX/YYY...` where `XX` is the first 2 hex chars of the SHA-1 hash.
- **Strict Format Compliance:** Objects are stored exactly as official Git expects: `"<type> <size>\0<content>"`, Zlib-compressed, hashed *before* compression.
- **Manual CLI Dispatching:** Arguments are parsed with `std::env::args()` to enforce explicit validation, safe indexing, and clean `cmd_*` function routing.
- **Rust-Native Memory Model:** Explicit ownership, `&[u8]` slicing, `Result`-based error propagation, and `Box<dyn Error>` for unified failure handling.

### 📦 Dependencies

Only three external crates are used, matching the blueprint's constraints:

```toml
[dependencies]
sha1 = "0.10"    # Cryptographic hashing
flate2 = "1.0"   # Zlib compression/decompression
hex = "0.4"      # Hex encoding utilities
```

## 🚀 Quick Start

```bash
# Build the project
cargo build --release

# Initialize a test repository
./target/release/git-rs init

# Store a file
echo "Hello Git Internals" > test.txt
./target/release/git-rs hash-object -w test.txt
# → b6fc4c620b67d95f953a5c1c1230aaab5db5a1b0

# Read it back
./target/release/git-rs cat-file -p b6fc4c620b67d95f953a5c1c1230aaab5db5a1b0
# → Hello Git Internals
```

## 🔍 Verification & Interoperability

Every phase is verified against the official `git` CLI. If official Git can read an object created by `git-rs`, the binary format is correct.

```bash
git cat-file -p <hash>   # Must print exact original content
git cat-file -t <hash>   # Must return "blob", "tree", or "commit"
git cat-file -s <hash>   # Must print content size in bytes
```

## 📚 Learning Objectives

- Understand how Git's DAG, content-addressable storage, and object database work
- Master Rust's ownership/borrowing system, `&[u8]` slices, and `?` error propagation
- Implement binary serialization protocols (null-byte separators, raw 20-byte hashes)
- Build a production-grade CLI with strict argument validation and clean dispatch routing

## 🗺️ Roadmap

| Phase | Feature | Status |
| ------- | --------- | -------- |
| 1 | `init` & `.git/` structure | ✅ Complete |
| 2 | `hash-object`, `cat-file` & object storage | ✅ Complete |
| 3 | `write-tree`, `ls-tree` & binary serialization | 🚧 In Progress |
| 4 | `commit-tree` & DAG parent references | 🔲 Planned |
| 5 | `commit` & refs/HEAD management | 🔲 Planned |
| 6 | `export-snapshot` & LLM Wiki integration | 🔲 Planned |

## 📖 Project Structure

```markdown
src/
├── main.rs          # CLI dispatcher & argument routing
├── object.rs        # SHA-1 hashing, Zlib compression, read/write objects
├── tree.rs          # Directory walking, tree serialization/parsing
├── commit.rs        # Commit metadata & DAG construction
└── refs.rs          # HEAD pointer & branch reference management
```

## 📜 License & Acknowledgments

Built following the [Build Git From Scratch in Rust](docs/build-git-in-rust.md) blueprint.  
*This project is a learning vehicle for systems programming and Rust fundamentals. Not intended for production use.*

```markdown

### 💡 Tips for Maintenance:
- Update the `Status` badge and `Roadmap` table as you complete each phase.
- The README is intentionally concise. It highlights *what* works, *how* to verify it, and *why* the architecture looks the way it does.
- When you finish Phase 3, simply change the tree/ls-tree row to `✅ Complete` and update the status badge.

Let me know when you're ready to start Checkpoint 3.1 (`TreeEntry` struct) and we'll move into Phase 3.
