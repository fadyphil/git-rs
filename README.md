<div align="center">

<img src="https://img.icons8.com/color/96/git.png" alt="git" align="middle"/>

# Git-rs

### Demystifying Version Control from First Principles

*A from-scratch implementation of Git's core object storage engine in Rust.*

<p align="center">
  <a href="https://skillicons.dev">
    <img src="https://skillicons.dev/icons?i=rust,git" alt="Tech Stack" />
  </a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Status-Phase_3_Complete-success?" alt="Status" />
  <img src="https://img.shields.io/badge/License-MIT-royalblue" alt="License" />
</p>

[Overview](#overview) • [Architecture](#architecture) • [Quick Start](#quick-start) • [Roadmap](#roadmap)

</div>

---

<a id="overview"></a>

## 📖 Overview

`git-rs` is not intended to replace Git. It is a surgical exploration of how version control actually works at the byte level.

By building Git's content-addressable storage, SHA-1 hashing, Zlib compression, and recursive tree serialization from first principles, this project strips away the magic and exposes the raw systems engineering underneath. It is a learning vehicle for mastering Rust's ownership model, binary serialization protocols, and Directed Acyclic Graph (DAG) traversal.

> **The North Star:** If the official, Linus Torvalds-authored Git binary can read, parse, and verify the objects created by `git-rs`, the implementation is correct.

---

<a id="features"></a>

## ✅ Implemented Features

| Command | Description | Engineering Concepts Mastered |
| :--- | :--- | :--- |
| `init` | Creates the `.git/` directory skeleton and `HEAD` pointer. | Filesystem I/O, Path resolution |
| `hash-object -w <file>` | Reads a file, constructs the Git blob format, computes SHA-1, compresses with Zlib, and stores it. | Byte buffers (`Vec<u8>`), Cryptographic hashing, Zlib streams |
| `cat-file <-p\|-t\|-s> <hash>` | Locates, decompresses, parses, and displays stored objects. | Binary parsing, Null-byte delimiters, UTF-8 coercion |
| `write-tree` | Snapshots the current directory into a binary tree object. | **Post-order DFS recursion**, Binary serialization, Raw 20-byte hashing |

---

<a id="architecture"></a>

## 🛠️ Architecture & Design

### The Byte-Level Contract

Git does not use JSON, XML, or high-level abstractions. It relies on a strict, continuous stream of bytes. `git-rs` respects this contract exactly:

```text
┌────────────────────────────────────────────────────────────┐
│  THE GIT OBJECT CONTRACT (In RAM before Zlib Compression)  │
├────────────────────────────────────────────────────────────┤
│  [ HEADER ]                                                │
│  "tree 74\0"  ◄── ASCII Text + Null Terminator             │
│                                                            │
│  [ BINARY PAYLOAD ]                                        │
│  "100644 README.md\0" + [20 Raw SHA-1 Bytes]               │
│  "040000 src\0"       + [20 Raw SHA-1 Bytes]               │
└────────────────────────────────────────────────────────────┘
```

### Core Systems Concepts

* **Content-Addressable Storage:** Every object is stored as `.git/objects/XX/YYY...` where `XX` is the first 2 hex chars of the SHA-1 hash. Deduplication is achieved by mathematical certainty, not heuristics.
* **Post-Order DAG Traversal:** Because a parent directory's hash is mathematically derived from its children, `write-tree` utilizes recursive post-order Depth-First Search to bubble hashes up the call stack.
* **Strict Format Compliance:** Objects are stored exactly as official Git expects: `"<type> <size>\0<content>"`, Zlib-compressed, and hashed *before* compression.
* **Rust-Native Memory Model:** Explicit ownership, `&[u8]` slice borrowing, `Result`-based error propagation, and `Box<dyn Error>` for unified failure handling. No garbage collection, no hidden allocations.

---

## 📦 Dependencies

To enforce a deep understanding of the standard library, external dependencies are strictly limited to the bare minimum required for cryptography and compression:

```toml
[dependencies]
sha1 = "0.10"    # Cryptographic hashing
flate2 = "1.0"   # Zlib compression/decompression
hex = "0.4"      # Hex encoding utilities
```

---

<a id="quick-start"></a>

## 🚀 Quick Start

```bash
# Clone and build the project
git clone <repo-url> && cd git-rs
cargo build --release

# Initialize a test repository
mkdir test-repo && cd test-repo
../target/release/git-rs init

# Store a file
echo "Hello Git Internals" > test.txt
../target/release/git-rs hash-object -w test.txt
# → b6fc4c620b67d95f953a5c1c1230aaab5db5a1b0

# Snapshot the directory
../target/release/git-rs write-tree
# → 4b825dc642cb6eb9a060e54bf8d69288fbee4904
```

---

## 🔍 Verification & Interoperability

Every phase is verified against the official `git` CLI. The ultimate test of interoperability:

```bash
# Read a tree object created by git-rs using the official Git binary
git cat-file -p <tree-hash>

# Expected Output:
# 100644 blob b6fc4c...    test.txt
```

If official Git can read the database, the binary format is mathematically correct.

---

<a id="roadmap"></a>

## 🗺️ Roadmap

| Phase | Feature | Status |
| :--- | :--- | :--- |
| **1** | `init` & `.git/` structure | ✅ **Complete** |
| **2** | `hash-object`, `cat-file` & object storage | ✅ **Complete** |
| **3** | `write-tree` & binary serialization | ✅ **Complete** |
| **4** | `commit-tree` & DAG parent references | 🔲 **In Progress** |
| **5** | `commit` & `refs/HEAD` management | 🔲 Planned |
| **6** | `export-snapshot` & LLM Wiki integration | 🔲 Planned |

---

## 📚 Project Structure

```text
src/
├── main.rs          # CLI dispatcher, argument routing, and command execution
├── object.rs        # SHA-1 hashing, Zlib compression, read/write objects
├── tree.rs          # Recursive directory walking, binary tree serialization
├── commit.rs        # (Next) Commit metadata, human-readable payloads, DAG construction
└── refs.rs          # (Next) HEAD pointer, branch reference management
```

---

## 🧠 Learning Objectives

This project is a deliberate exercise in systems programming:

1. **Memory & Ownership:** Master Rust's borrow checker, `&[u8]` slices, and zero-copy parsing.
2. **Binary Protocols:** Implement strict serialization (null-byte separators, raw 20-byte hashes vs 40-char hex strings).
3. **Graph Theory:** Understand how Directed Acyclic Graphs (DAGs) enforce history integrity and enable deduplication.
4. **CLI Architecture:** Build a production-grade dispatcher with strict argument validation and clean error propagation.

---

<div align="center">

*Built following the [Build Git From Scratch in Rust](docs/build-git-in-rust.md) blueprint.*
*This project is a learning vehicle for systems programming. Not intended for production use.*

</div>
