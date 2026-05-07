---
banner: pics/git.jpg
---
# Build Git From Scratch in Rust

## A Deep Learning Blueprint — No Detail Too Small

> **How to use this document.**
> Read it sequentially. Every block builds on the one before it. Do not skip sections. The questions embedded throughout are not rhetorical — stop and answer them in your head or on paper before continuing. If you cannot answer a question, re-read the section above it. The inevitability of learning is enforced by the structure of the document itself.

---

## Table of Contents

- [Part 0 — Before You Begin](#part-0---before-you-begin)
- [Part 1 — Rust Prerequisites](#part-1---rust-prerequisites)
- [Part 2 — What Git Actually Is](#part-2---what-git-actually-is)
- [Part 3 — The Object Format Bible](#part-3---the-object-format-bible)
- [Part 4 — The Project Blueprint](#part-4---the-project-blueprint)
- [Part 5 — Phase-by-Phase Implementation](#part-5---phase-by-phase-implementation)
- [Part 6 — The LLM Wiki Extension](#part-6---the-llm-wiki-extension)
- [Appendix A — Questions for Deep Retention](#appendix-a---questions-for-deep-retention)
- [Appendix B — Verification Tests Per Phase](#appendix-b---verification-tests-per-phase)
- [The North Star](#the-north-star)

---

# Part 0 - Before You Begin

## 0.1 The Philosophy of This Project

You are not building Git because you need a version control system. You are building Git because the act of building it will rewrite how you think about data, memory, and systems. Every concept in this project — hashing, compression, content-addressable storage, the DAG — is a concept that transfers directly into databases, operating systems, distributed systems, and AI infrastructure.

The reason Git specifically is chosen as the learning vehicle is this: Git is one of the smallest complete systems in existence that simultaneously demonstrates file I/O, cryptographic hashing, binary serialization, tree structures, and a clean command-line interface architecture. It is a complete universe in a small box.

The reason Rust is the language is this: Dart and every garbage-collected language you have ever used hides two things from you. First, it hides *ownership* — who is responsible for a piece of memory and when that memory is freed. Second, it hides *bytes* — strings are not byte arrays, objects are not memory addresses, everything is abstracted into comfortable objects. Rust removes both abstractions by force. The compiler will refuse to compile code that doesn't think clearly about ownership and bytes. This is not a punishment. It is a curriculum.

---

## 0.2 What You Will Be Able To Do At The End

When you finish Phase 5 of this document, you will have a Rust binary that can:

1. Initialize a new Git repository (`.git/` directory structure)
2. Store any file as a compressed, SHA-1-hashed blob object
3. Read any stored blob back and decompress it
4. Build a tree object from a directory snapshot
5. Create a commit object that references a tree
6. Read the commit history

When you finish Phase 6, you will have a translation layer that reads your custom Git DAG and outputs structured text (JSON or Markdown) that a local LLM can consume to maintain a living knowledge base of your codebase — Karpathy's LLM Wiki pattern, powered by your own storage engine.

---

## 0.3 Prerequisites Checklist

Before reading Part 1, verify that you have:

- [x] Rust installed: `rustup` and `cargo` working ✅ 2026-05-03
- [x] `cargo new git-rs --bin` creates a new binary project ✅ 2026-05-03
- [x] You can run `cargo run` in a new project and see "Hello, world!" ✅ 2026-05-03
- [x] You have read the first three chapters of [The Rust Book](https://doc.rust-lang.org/book/) — if not, read them now. This document assumes basic syntax familiarity but explains all deep concepts from first principles. ✅ 2026-05-03

---

<a name="part-1"></a>

# Part 1 - Rust Prerequisites

> This is not a Rust tutorial. This is a targeted briefing on exactly the Rust concepts you will hit in this project, explained in terms you already know from Dart and object-oriented thinking. Read every subsection even if you think you know it.

---

## 1.1 Ownership — The Central Idea

In Dart, this is perfectly legal:

```dart
String a = "hello";
String b = a;        // Both a and b exist. Both point to "hello".
print(a);            // Fine.
print(b);            // Fine.
```

The Dart VM's garbage collector keeps "hello" alive as long as either `a` or `b` references it. You never think about who "owns" the string. The GC does.

In Rust, this is the equivalent:

```rust
let a = String::from("hello");
let b = a;           // Ownership of "hello" is MOVED to b.
println!("{}", a);   // COMPILER ERROR: a no longer owns anything.
println!("{}", b);   // Fine.
```

This is not a quirk. This is the entire system. Rust enforces a rule: **every piece of data has exactly one owner at any point in time.** When the owner goes out of scope, the data is freed. No GC needed.

**Why this matters for Git:** When you read a file into memory, you get a `Vec<u8>` (a heap-allocated byte array). You will need to pass those bytes to a SHA-1 hasher AND to a Zlib compressor. If you try to pass ownership twice, the compiler will refuse. You will need to understand borrowing (next section) to solve this.

> **❓ Question 1.1:** In Dart, what happens to an object when the last reference to it is removed? Who does that cleanup? In Rust, what happens to a value when its owner goes out of scope?

---

## 1.2 Borrowing & References

The solution to needing to use a value without consuming it is *borrowing*. You lend a value to a function via a reference. The owner keeps ownership.

```rust
fn print_length(data: &Vec<u8>) {  // Takes a reference, not ownership
    println!("{}", data.len());
    // data is a borrowed reference. We cannot move or drop it here.
}

fn main() {
    let bytes = vec![1u8, 2, 3, 4];
    print_length(&bytes);  // Lend bytes to the function
    print_length(&bytes);  // Lend it again — still valid, we never gave it away
    // bytes is still owned here. It will be freed when main() ends.
}
```

There are two kinds of references:

**Shared (immutable) reference: `&T`**

- Multiple `&T` references can exist at the same time
- You can read through them, never write
- Analogy: multiple people reading the same document simultaneously

**Exclusive (mutable) reference: `&mut T`**

- Only ONE `&mut T` can exist at a time
- No `&T` can exist at the same time as a `&mut T`
- You can both read and write through it
- Analogy: one person editing a document while everyone else is locked out

```rust
let mut data = vec![1u8, 2, 3];

let r1 = &data;      // Shared borrow — OK
let r2 = &data;      // Another shared borrow — OK
// let r3 = &mut data; // ERROR: Cannot have &mut while &T exists

drop(r1);  // Release the shared borrows
drop(r2);

let r3 = &mut data;  // Now we can take an exclusive borrow
r3.push(4);
```

**Why this matters for Git:** The SHA-1 hasher and Zlib compressor both need to read your byte buffer. Since reading doesn't mutate, you can pass `&[u8]` (a shared byte slice reference) to both of them sequentially.

> **❓ Question 1.2:** If you have a `Vec<u8>` called `buffer`, how do you pass it to two different functions without either function consuming it?

---

## 1.3 Slices — `&[u8]` and `&str`

A slice is a *view* into a contiguous sequence of data. It does not own the data. It is just a pointer + a length.

```
 Vec<u8> in memory:
 ┌──────────────────────────────────────┐
 │ ptr → heap │ len: 5 │ capacity: 8   │
 └──────────────────────────────────────┘
                  ↓
           [ 0x62, 0x6c, 0x6f, 0x62, 0x00 ]  ← actual bytes on heap

 &[u8] slice:
 ┌──────────────────┐
 │ ptr → heap │ len │     ← points INTO the Vec's data, no copy
 └──────────────────┘
```

`&[u8]` means "a borrowed reference to a sequence of bytes." When a function takes `&[u8]`, it accepts slices from Vecs, arrays, or anything that is a contiguous byte sequence.

`&str` is exactly the same thing but the bytes are guaranteed to be valid UTF-8. It is literally `&[u8]` with a UTF-8 contract.

```rust
let owned: Vec<u8> = vec![104, 101, 108, 108, 111];
let view: &[u8] = &owned;       // Borrow the whole Vec as a slice
let partial: &[u8] = &owned[1..3]; // Borrow bytes at index 1 and 2 only
```

**Why this matters for Git:** The `sha1` crate's `update()` method takes `&[u8]`. The `flate2` compressor writes from `&[u8]`. You will write code like this constantly:

```rust
hasher.update(&content);         // Pass a view of content to the hasher
encoder.write_all(&content)?;    // Pass a view of content to the compressor
```

> **❓ Question 1.3:** What is the difference between `Vec<u8>` and `&[u8]`? Which one allocates memory on the heap? Which one is just a view into existing data?

---

## 1.4 Structs

You know classes from Dart. Rust has structs instead. Structs hold data. Methods are attached separately via `impl` blocks.

```rust
// Define the struct — data only
struct GitObject {
    kind: String,
    content: Vec<u8>,
}

// Attach methods via impl
impl GitObject {
    // Associated function (no `self`) — like a static factory method
    fn new(kind: &str, content: Vec<u8>) -> Self {
        GitObject {
            kind: kind.to_string(),
            content,
        }
    }

    // Method (takes &self) — read-only access to fields
    fn size(&self) -> usize {
        self.content.len()
    }

    // Method (takes &mut self) — mutable access to fields
    fn append(&mut self, byte: u8) {
        self.content.push(byte);
    }
}
```

No `new` keyword. No `class`. No inheritance. Composition through embedding structs in other structs.

> **❓ Question 1.4:** In Dart, you write `class Foo { Foo(); void bar() {} }`. What is the Rust equivalent? Where do methods live relative to the data?

---

## 1.5 Enums & Pattern Matching

Rust's enums are not Java/Dart enums. They are *algebraic data types* — each variant can carry different data.

```rust
enum Command {
    Init,                          // No data
    HashObject { path: String },   // Carries a path
    CatFile { hash: String },      // Carries a hash
    WriteTree,
}
```

You handle enums with `match` — Rust's switch statement that is guaranteed to be exhaustive:

```rust
fn run(cmd: Command) {
    match cmd {
        Command::Init => init_repo(),
        Command::HashObject { path } => hash_file(path),
        Command::CatFile { hash } => read_object(hash),
        Command::WriteTree => write_tree(),
    }
    // If you forget a variant, the compiler refuses to compile.
}
```

This is how you will model every command your Git CLI accepts. The `match` statement is the architectural backbone of the CLI dispatcher.

You already know this pattern from `freezed` sealed classes and `when()` in Dart — this is the same idea, baked into the language.

> **❓ Question 1.5:** What happens in Rust if you write a `match` on an enum but forget to handle one variant? What does the compiler do?

---

## 1.6 Error Handling — `Result<T, E>` and `Option<T>`

There are no exceptions in Rust. Period. Every operation that can fail returns a `Result<T, E>`.

```rust
// Result<T, E> — either a success value of type T, or an error of type E
enum Result<T, E> {
    Ok(T),   // Success — carries the value
    Err(E),  // Failure — carries the error
}

// Option<T> — either a value of type T, or nothing
enum Option<T> {
    Some(T),  // Has a value
    None,     // Has no value
}
```

Every file operation, every network call, every parse operation returns a `Result`. You must decide what to do with errors. The compiler will warn you if you silently ignore them.

```rust
use std::fs;

// Reading a file returns Result<String, std::io::Error>
let contents = fs::read_to_string("somefile.txt");

match contents {
    Ok(text) => println!("File: {}", text),
    Err(e) => eprintln!("Failed: {}", e),
}
```

**The `?` operator** — propagates errors upward automatically:

```rust
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;  // If Err, return that Err immediately
    Ok(contents)  // If we get here, it worked
}
```

The `?` operator is equivalent to:

```rust
let contents = match fs::read_to_string(path) {
    Ok(val) => val,
    Err(e) => return Err(e),
};
```

In your Git implementation, *every single function* will return `Result<Something, Box<dyn std::error::Error>>`. The `?` will be your most-used operator.

`Box<dyn std::error::Error>` means "a heap-allocated value that implements the Error trait" — it lets you return any type of error from a function without specifying exactly which error type. It is the pragmatic escape hatch for a CLI tool.

> **❓ Question 1.6:** In Dart, if `File.readAsString()` throws, what happens? In Rust, if `fs::read_to_string()` fails, what does it return? How does the `?` operator save you from writing `match` on every single call?

---

## 1.7 Traits

A trait is an interface. It defines a set of methods that a type must implement. You declare that a type implements a trait with the `impl Trait for Type` syntax.

```rust
trait Serialize {
    fn to_bytes(&self) -> Vec<u8>;
}

struct BlobObject {
    content: Vec<u8>,
}

impl Serialize for BlobObject {
    fn to_bytes(&self) -> Vec<u8> {
        // Construct the Git blob format: "blob <size>\0<content>"
        let header = format!("blob {}\0", self.content.len());
        let mut result = header.into_bytes();
        result.extend_from_slice(&self.content);
        result
    }
}
```

The critical Rust standard library traits you will use:

- **`std::fmt::Display`** — lets a type be printed with `{}`
- **`std::io::Write`** — lets a type be written to (files, byte buffers, the compressor)
- **`std::io::Read`** — lets a type be read from
- **`std::error::Error`** — marks a type as an error

> **❓ Question 1.7:** In Dart, you use `implements` for interfaces. In Rust, you use `impl Trait for Struct`. What is the difference between a Rust `trait` and inheritance? Why does Rust not have inheritance?

---

## 1.8 The `impl Trait` in Function Parameters

You can use traits as parameter types, meaning "accept any type that implements this trait":

```rust
use std::io::Write;

// This function accepts anything that implements Write:
// a file, a Vec<u8>, a network socket, a Zlib encoder — all work
fn write_content(writer: &mut impl Write, data: &[u8]) -> Result<(), std::io::Error> {
    writer.write_all(data)
}
```

This is how you write functions that work with the Zlib encoder — you pass it as an `impl Write` and write bytes into it.

---

## 1.9 Vec<u8> — Your Byte Buffer

`Vec<u8>` (a vector of unsigned 8-bit integers, i.e., bytes) is the most important data structure in this project. Every file you read will start as a `Vec<u8>`. Every object you write to disk will end as a `Vec<u8>`.

```rust
// Creating
let mut buffer: Vec<u8> = Vec::new();  // Empty, grows on heap

// Converting a String to bytes
let header = format!("blob 42\0");    // A String
let header_bytes: Vec<u8> = header.into_bytes();  // Consume String, get Vec<u8>

// Appending one Vec to another
buffer.extend_from_slice(&header_bytes);  // Append without consuming

// Writing a literal byte
buffer.push(0x00);  // Append a single null byte

// Getting a slice view
let view: &[u8] = &buffer;  // Borrow the whole buffer as a slice
let partial: &[u8] = &buffer[0..4];  // View first 4 bytes
```

**Converting between bytes and hex strings** (you will need this for SHA-1 hashes):

```rust
// A SHA-1 hash is 20 raw bytes. To display it, you hex-encode it:
let hash_bytes: [u8; 20] = [0xda, 0x39, 0xa3, /* ... */];
let hex_string: String = hash_bytes
    .iter()
    .map(|b| format!("{:02x}", b))  // Each byte → 2-char hex
    .collect::<String>();           // Collect all chars into one String
// Result: "da39a3..."
```

> **❓ Question 1.8:** Why is a SHA-1 hash represented as 20 bytes but displayed as a 40-character string? What is the relationship between a byte (0-255) and two hexadecimal characters?

---

## 1.10 Path & PathBuf

For file system paths, Rust has two types analogous to `String`/`&str`:

- `PathBuf` — owned, heap-allocated path (like `String`)
- `&Path` — borrowed view of a path (like `&str`)

```rust
use std::path::{Path, PathBuf};

// Creating a PathBuf
let root = PathBuf::from(".git");

// Joining paths
let objects_dir = root.join("objects");  // → ".git/objects"

// Checking existence
if objects_dir.exists() { /* ... */ }

// Converting to a string (can fail if path contains non-UTF8)
let s = objects_dir.to_str().unwrap();  // Returns &str, panics if non-UTF8

// Getting the parent directory
let parent = objects_dir.parent();  // Returns Option<&Path>
```

---

## 1.11 File I/O with `std::fs`

The operations you will use most:

```rust
use std::fs;
use std::io::Write;

// Read entire file into Vec<u8>
let bytes: Vec<u8> = fs::read("path/to/file")?;

// Read entire file into String
let text: String = fs::read_to_string("path/to/file")?;

// Create directory (and all parent dirs)
fs::create_dir_all(".git/objects/ab")?;

// Write Vec<u8> to file
fs::write(".git/objects/ab/cdef...", &compressed_bytes)?;

// Open a file for writing (more control)
use std::fs::File;
let mut file = File::create("path")?;
file.write_all(&data)?;

// Open a file for reading
let mut file = File::open("path")?;
// then use a reader on it
```

---

## 1.12 CLI Argument Parsing

For a minimal CLI, `std::env::args()` is sufficient. It returns an iterator over the command-line arguments as `String`s.

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    // args[0] is always the program name
    // args[1] is the first argument
    // args[2] is the second, etc.

    if args.len() < 2 {
        eprintln!("Usage: git-rs <command>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "init" => cmd_init(),
        "hash-object" => {
            if args.len() < 4 || args[2] != "-w" {
                eprintln!("Usage: git-rs hash-object -w <file>");
                std::process::exit(1);
            }
            cmd_hash_object(&args[3]).unwrap();
        }
        "cat-file" => {
            cmd_cat_file(&args[2], &args[3]).unwrap();
        }
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            std::process::exit(1);
        }
    }
}
```

> **❓ Question 1.9:** `args[1].as_str()` — why do you call `.as_str()` before matching? What type is `args[1]`? What type does `match` need for string literals?

---

## 1.13 The `?` Operator and Error Propagation Chain

By the time you finish this project, your main function will look like this:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "init" => cmd_init()?,
        "hash-object" => cmd_hash_object(&args[3])?,
        "cat-file" => cmd_cat_file(&args[2], &args[3])?,
        _ => {}
    }
    Ok(())
}
```

Every `cmd_*` function returns `Result<(), Box<dyn Error>>`. The `?` at the call site means: "if this returns `Err`, print the error and exit." The `Box<dyn Error>` in `main`'s return type means: if an error reaches `main`, Rust will automatically print it and exit with a non-zero code.

---

<a name="part-2"></a>

# Part 2 - What Git Actually Is

> Stop thinking about Git as a version control system. Start thinking about it as a database. Specifically: a content-addressable, append-only, compressed object database.

---

## 2.1 The Mental Model

Here is the core insight that makes everything else make sense:

**Git does not track file names. Git tracks content.**

When you run `git add file.txt`, Git does not say "I will remember that `file.txt` exists." Git says "I will store the *bytes inside* `file.txt` and give those bytes a unique ID based on their content."

If you create `file_copy.txt` with the exact same content as `file.txt`, Git will store only ONE copy of those bytes. Both files map to the same stored object. This is content-addressable storage.

The unique ID for any piece of content is its SHA-1 hash. More on that in a moment.

```
Your working directory:          Git's object database:
─────────────────────            ─────────────────────────────────────────
file.txt       (5000 bytes)  →   da39a3ee5e6b4b0d3255bfef95601890afd80709
file_copy.txt  (5000 bytes)  →   da39a3ee5e6b4b0d3255bfef95601890afd80709
                                              ↑
                            same hash because same content = stored once
```

> **❓ Question 2.1:** If two different files have the same content, how many objects does Git store? What uniquely identifies an object in Git's database?

---

## 2.2 The .git Directory — A Complete Tour

When you run `git init`, Git creates this structure:

```
.git/
├── objects/          ← THE DATABASE. All stored objects live here.
│   ├── info/         ← (Empty in a fresh repo, used for pack files later)
│   └── pack/         ← (Empty in a fresh repo, used for pack files later)
├── refs/             ← Named pointers to commit hashes
│   ├── heads/        ← Local branch pointers (e.g., refs/heads/main)
│   └── tags/         ← Tag pointers
├── HEAD              ← Pointer to the currently checked-out branch
├── config            ← Repository configuration
├── description       ← Used by GitWeb, ignore it
└── info/
    └── exclude       ← Like .gitignore but not shared
```

The `objects/` directory is the entire storage engine. Everything else is metadata.

An object is stored at a path derived from its hash:

```
Object hash: da39a3ee5e6b4b0d3255bfef95601890afd80709
                ↓↓
Stored at:  .git/objects/da/39a3ee5e6b4b0d3255bfef95601890afd80709
                         ^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                    first 2    remaining 38 characters = filename
                    chars = directory name
```

Why split the hash into directory + filename? Pure performance. If you stored all objects in one flat directory, a repository with 100,000 objects would have a directory with 100,000 entries — filesystem operations on large directories are slow. Splitting by the first 2 hex characters creates at most 256 subdirectories (00 through ff), each with far fewer entries.

> **❓ Question 2.2:** A SHA-1 hash is 40 hex characters. Git uses the first 2 as a directory and the remaining 38 as a filename. Why does splitting into directories improve performance?

---

## 2.3 SHA-1 Hashing — What It Is

SHA-1 is a *cryptographic hash function*. It takes an input of any size and produces a fixed-size output: 20 bytes, displayed as 40 hexadecimal characters.

Properties that matter for Git:

1. **Deterministic:** The same input always produces the same output. Run SHA-1 on "hello" today and tomorrow, you get the same hash.
2. **Avalanche Effect:** A tiny change in input produces a completely different output. Change one bit of "hello" and the hash is unrecognizable.
3. **One-way:** You cannot reverse a hash back to its input. Given `da39a3...`, you cannot reconstruct what was hashed.
4. **Collision-resistant (practically):** It is practically impossible to find two different inputs that produce the same hash.

```
Input:    "blob 5\0hello"     (Git's format for storing "hello")
SHA-1:    b6fc4c620b67d95f953a5c1c1230aaab5db5a1b0

Input:    "blob 5\0hellp"     (One character changed)
SHA-1:    completely different hash
```

SHA-1 has known cryptographic weaknesses (SHAttered attack, 2017), which is why modern Git supports SHA-256. For your implementation, SHA-1 is correct and will interoperate with the official Git CLI.

**In Rust, using the `sha1` crate:**

```rust
use sha1::{Sha1, Digest};

let mut hasher = Sha1::new();
hasher.update(b"blob 5\0hello");  // Feed bytes to the hasher
let result = hasher.finalize();   // Consume the hasher, get the 20-byte result
// result is a GenericArray<u8, U20> — treat it like [u8; 20]
```

---

## 2.4 Zlib Compression — What It Is

Zlib is a lossless data compression algorithm. Git compresses every object before writing it to disk. A 1MB source file stored as a Git blob might compress to 400KB. Compression is applied to the formatted object header + content.

```
What goes INTO the compressor:    "blob 11\0hello world"
What comes OUT of the compressor: [compressed binary bytes — unreadable by human eyes]
```

Zlib compression is completely reversible. Given the compressed bytes, you can reconstruct the original exactly.

**In Rust, using the `flate2` crate:**

```rust
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::Write;

// Compressing
let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
encoder.write_all(&raw_bytes)?;          // Feed bytes into the encoder
let compressed = encoder.finish()?;      // Flush and get the compressed Vec<u8>

// Decompressing
use flate2::read::ZlibDecoder;
use std::io::Read;

let mut decoder = ZlibDecoder::new(&compressed[..]);
let mut decompressed = Vec::new();
decoder.read_to_end(&mut decompressed)?;
```

The `ZlibEncoder` takes a `Vec::new()` as its "writer" — it compresses into that Vec. The `ZlibDecoder` takes a byte slice and reads decompressed bytes out of it.

> **❓ Question 2.3:** Git hashes the data BEFORE compressing it. Why does the hash have to be computed before compression? (Hint: would the compressed bytes always be identical?)

---

## 2.5 The Four Object Types

Git stores four types of objects. Every object, regardless of type, shares the same outer structure: a header (`type size\0`) followed by the content. They differ only in what the "content" section contains.

```
┌─────────────────────────────────────────────────────┐
│  EVERY GIT OBJECT ON DISK                           │
│                                                     │
│  Zlib-compressed bytes containing:                  │
│  ┌────────────────────────────────────────────────┐ │
│  │  "<type> <size>\0<content bytes>"              │ │
│  └────────────────────────────────────────────────┘ │
│                                                     │
│  The SHA-1 of the UNCOMPRESSED bytes is the hash.   │
└─────────────────────────────────────────────────────┘
```

**Object type 1: Blob**
Stores the raw content of a file. Nothing but the bytes of the file. No filename, no permissions — just bytes.

**Object type 2: Tree**
Stores a directory snapshot. A list of entries, each saying: "this filename, at these permissions, points to this object hash." Each entry points to either a blob (file) or another tree (subdirectory).

**Object type 3: Commit**
Stores a snapshot of the project at a point in time. Contains: the hash of the root tree, the hash of the parent commit (if any), author info, timestamp, and the commit message.

**Object type 4: Tag**
Stores a named, annotated reference to another object (usually a commit). You will not implement this in the initial phases.

---

## 2.6 The Directed Acyclic Graph (DAG)

Git's history is a Directed Acyclic Graph. Every commit points to its parent commit(s). The edges point backward in time — a commit knows its parents, but a parent has no knowledge of its children.

```
  [Commit C]  ←—  [Commit D]   ←—  [Commit E] (HEAD)
      │                 │                 │
      ↓                 ↓                 ↓
  [Tree 1]          [Tree 2]          [Tree 3]
   /     \           /     \           /     \
[Blob A] [Blob B] [Blob A] [Blob C] [Blob D] [Blob C]
                    ↑
         Notice: Blob A appears in Commits C and D
         because that file didn't change.
         One copy on disk, referenced twice.
```

*Directed:* Edges have a direction (child → parent)
*Acyclic:* You can never follow edges and arrive back where you started. History never loops. (This would mean a commit was its own ancestor — logically impossible.)

This structure means:

- **Integrity:** You cannot modify a past commit without changing its hash, which changes every descendant's hash. Tampering is detectable.
- **Efficiency:** Unchanged files are never stored twice, even across commits.
- **Correctness:** Any state of the repository can be identified by a single commit hash.

> **❓ Question 2.4:** If you edit one line in one file and make a new commit, which objects need to change? (Hint: think about which objects reference which other objects and how the hashes propagate.)

---

<a name="part-3"></a>

# Part 3 - The Object Format Bible

> This is the reference section. The byte-for-byte format of every object you will implement. Return to this section every time you are implementing a new command.

---

## 3.1 Blob Object Format

The simplest object. Pure file content, wrapped in a tiny header.

```
┌──────────────────────────────────────────────────────────┐
│  BLOB FORMAT (uncompressed, before SHA-1 and Zlib)       │
│                                                          │
│  "blob " + decimal(content_length) + "\0" + content      │
│                                                          │
│  Example: Storing the file containing "hello\n" (6 bytes)│
│                                                          │
│  Bytes: 62 6c 6f 62 20 36 00 68 65 6c 6c 6f 0a           │
│         b  l  o  b  SP 6  NUL h  e  l  l  o  LF          │
│                                                          │
│  Process:                                                │
│  1. Read file bytes into Vec<u8>                         │
│  2. Format header: "blob {}\0".format(content.len())     │
│  3. Concatenate: header_bytes + content_bytes            │
│  4. SHA-1(header + content) → 40-char hex hash           │
│  5. Zlib-compress(header + content) → compressed bytes   │
│  6. Write compressed bytes to .git/objects/XX/YYY...     │
└──────────────────────────────────────────────────────────┘
```

**The null byte (`\0`) is critical.** It separates the header from the content. The `\0` is byte value 0x00. In Rust: `b'\0'` or `0u8`. In a format string: `"\0"`.

---

## 3.2 Tree Object Format

A tree object stores a directory snapshot. Its content is a binary-encoded list of entries. Each entry encodes one file or subdirectory.

```
┌──────────────────────────────────────────────────────────────────┐
│  TREE FORMAT (uncompressed content, after "tree <size>\0")       │
│                                                                  │
│  Repeated for each entry:                                        │
│  "<mode>" + " " + "<filename>" + "\0" + <20 raw hash bytes>      │
│                                                                  │
│  mode is an ASCII string:                                        │
│    "100644"  → regular file (non-executable)                     │
│    "100755"  → executable file                                   │
│    "040000"  → directory (note: leading zero)                    │
│    "120000"  → symbolic link                                     │
│                                                                  │
│  The hash is 20 RAW BYTES — NOT the 40-char hex string.          │
│  This is the crucial difference from the header section.         │
│                                                                  │
│  Example entry for file "hello.txt" with hash da39a3...:         │
│  "100644 hello.txt\0" + [0xda, 0x39, 0xa3, ... 20 bytes total]   │
└──────────────────────────────────────────────────────────────────┘
```

**The raw 20-byte hash.** This is the most common mistake. When storing tree entries, you store the 20 binary bytes of the SHA-1 hash, not the 40 hex characters. You must convert hex → binary.

```rust
// Converting 40-char hex hash string → 20 raw bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i+2], 16).unwrap())
        .collect()
}
// "da39a3" → [0xda, 0x39, 0xa3]
```

The full tree content (before header) is all entries concatenated:

```
entry_1_bytes + entry_2_bytes + entry_3_bytes + ...
```

Then you wrap it with the tree header exactly as you do for blobs:

```
"tree <size_of_all_entries_concatenated>\0" + all_entries
```

> **❓ Question 3.1:** Why does the tree format use raw 20-byte hashes for its entries instead of 40-char hex strings? (Hint: think about space — how much space does a raw 20-byte hash use vs. a 40-char hex string?)

---

## 3.3 Commit Object Format

A commit object's content is human-readable text, unlike the binary tree format.

```
┌──────────────────────────────────────────────────────────────────┐
│  COMMIT FORMAT (uncompressed content, after "commit <size>\0")   │
│                                                                  │
│  "tree <40-char-hex-tree-hash>\n"                                │
│  "parent <40-char-hex-parent-commit-hash>\n"  ← omit for 1st     │
│  "author <name> <<email>> <unix-timestamp> <timezone>\n"         │
│  "committer <name> <<email>> <unix-timestamp> <timezone>\n"      │
│  "\n"                                                            │
│  "<commit message>\n"                                            │
│                                                                  │
│  Example:                                                        │
│  "tree da39a3ee5e6b4b0d3255bfef95601890afd80709\n"               │
│  "author Fady <fady@email.com> 1714601234 +0200\n"               │
│  "committer Fady <fady@email.com> 1714601234 +0200\n"            │
│  "\n"                                                            │
│  "Initial commit\n"                                              │
└──────────────────────────────────────────────────────────────────┘
```

The unix timestamp is seconds since January 1, 1970 UTC. In Rust: `std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()`.

The timezone offset is your local UTC offset as a string: `+0200` for Egypt (UTC+2).

Note: unlike the tree format, commit objects use the 40-char hex hash in their `tree` and `parent` lines — not raw bytes. The commit format is text; the tree format is binary. Be deliberate about which you use where.

---

## 3.4 Refs and HEAD

`HEAD` is the pointer that tells Git "which branch are you currently on?" It is a plain text file.

```
.git/HEAD  contains:  "ref: refs/heads/main\n"
```

This means HEAD does not point directly to a commit hash. It points to a *reference* (a branch name). The reference is itself a file:

```
.git/refs/heads/main  contains:  "da39a3ee5e6b4b0d3255bfef95601890afd80709\n"
```

So to find the current commit: read HEAD → get branch name → read the branch file → get the commit hash.

When you make a new commit, you must update the branch file to point to the new commit hash. HEAD itself does not change (it still points to the same branch name).

```
  HEAD file           refs/heads/main file         commit object in objects/
  ─────────────       ────────────────────         ──────────────────────────
  "ref: refs/         "da39a3...\n"           →    commit da39a3...
   heads/main\n"          │                          tree: ...
       │                  └──────────────────→        parent: ...
       └──────────────────────────────────────        message: ...
```

---

<a name="part-4"></a>

# Part 4 - The Project Blueprint

---

## 4.1 Cargo.toml — Your Dependencies

```toml
[package]
name = "git-rs"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "git-rs"
path = "src/main.rs"

[dependencies]
sha1 = "0.10"        # SHA-1 hashing
flate2 = "1.0"       # Zlib compression/decompression
hex = "0.4"          # Hex encoding/decoding utilities
```

These three crates are the only non-standard-library dependencies you need for Phases 1–6. Everything else — file I/O, path handling, CLI parsing, string formatting — comes from Rust's standard library. This is intentional. Using the standard library forces you to understand the language itself.

---

## 4.2 Directory Structure

```
git-rs/
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── main.rs            ← CLI entry point and command dispatcher
    ├── object.rs          ← Core: reading/writing/hashing Git objects
    ├── store.rs           ← Filesystem operations (.git/objects/ I/O)
    ├── tree.rs            ← Tree object construction and parsing
    ├── commit.rs          ← Commit object construction and parsing
    └── refs.rs            ← HEAD and refs/ operations
```

You will build this module by module. Do not create all files at once. Create each file when you reach its corresponding phase.

---

## 4.3 Module Architecture

```
main.rs
  │
  ├── parses args
  ├── dispatches to cmd_* functions
  │
  ├──→ object.rs
  │       │  GitObject struct
  │       │  hash_content() → String
  │       │  read_object() → GitObject
  │       │  write_object() → String (returns hash)
  │       │
  │       └──→ store.rs
  │               │  object_path() → PathBuf
  │               │  read_raw()  → Vec<u8>
  │               │  write_raw() → ()
  │
  ├──→ tree.rs
  │       │  TreeEntry struct
  │       │  build_tree_from_dir() → String (returns tree hash)
  │       │  parse_tree() → Vec<TreeEntry>
  │
  ├──→ commit.rs
  │       │  CommitData struct
  │       │  create_commit() → String (returns commit hash)
  │       │  parse_commit() → CommitData
  │
  └──→ refs.rs
          │  read_head() → String (branch name or hash)
          │  read_ref() → String (commit hash)
          │  write_ref() → ()
          │  update_head() → ()
```

---

## 4.4 Data Flow Diagram — `hash-object -w file.txt`

This is the complete pipeline for the most important command. Every other command is a variation on this flow.

```
  ┌─────────────────────────────────────────────────────────────────────┐
  │                                                                     │
  │  $ git-rs hash-object -w hello.txt                                  │
  │                                                                     │
  │  1. READ                                                            │
  │     fs::read("hello.txt")                                           │
  │     → content: Vec<u8>  [104,101,108,108,111,10]  ("hello\n")       │
  │                                                                     │
  │  2. BUILD HEADER                                                    │
  │     format!("blob {}\0", content.len())                             │
  │     → header: "blob 6\0"                                            │
  │     → header_bytes: [98,108,111,98,32,54,0]                         │
  │                                                                     │
  │  3. CONCATENATE                                                     │
  │     header_bytes + content                                          │
  │     → full_object: [98,108,111,98,32,54,0,104,101,108,108,111,10]   │
  │                                                                     │
  │  4. HASH (SHA-1 of full_object)                                     │
  │     → hash_bytes: [0x8c, 0x7e, 0x5a, ...]  (20 bytes)               │
  │     → hash_hex: "8c7e5a..."  (40 chars)                             │
  │                                                                     │
  │  5. COMPRESS (Zlib of full_object)                                  │
  │     → compressed: Vec<u8>  (smaller than full_object)               │
  │                                                                     │
  │  6. WRITE TO DISK                                                   │
  │     dir: .git/objects/8c/                                           │
  │     file: .git/objects/8c/7e5a...  (remaining 38 chars)             │
  │     content: compressed bytes                                       │
  │                                                                     │
  │  7. PRINT                                                           │
  │     stdout: "8c7e5a..."                                             │
  │                                                                     │
  └─────────────────────────────────────────────────────────────────────┘
```

---

<a name="part-5"></a>

# Part 5 - Phase-by-Phase Implementation

> Each phase ends with a verification test. Do not proceed to the next phase until the verification test passes. The tests use the official `git` CLI to verify your output — if git can read what you wrote, your format is correct.

---

## Phase 1 — `init`

**Goal:** Create the `.git` directory skeleton.

**What you're building:**

```
Running: $ git-rs init

Creates:
  .git/
  .git/objects/
  .git/objects/info/
  .git/objects/pack/
  .git/refs/
  .git/refs/heads/
  .git/refs/tags/
  .git/HEAD            ← contains "ref: refs/heads/main\n"

Prints: "Initialized empty Git repository in .git/"
```

**The code — `src/main.rs`:**

```rust
use std::fs;
use std::io::Write;

fn cmd_init() -> Result<(), Box<dyn std::error::Error>> {
    // Create all required directories
    // create_dir_all creates parent dirs automatically — like `mkdir -p`
    fs::create_dir_all(".git/objects/info")?;
    fs::create_dir_all(".git/objects/pack")?;
    fs::create_dir_all(".git/refs/heads")?;
    fs::create_dir_all(".git/refs/tags")?;

    // Write the HEAD file
    // Note: "ref: refs/heads/main\n" — the trailing newline matters
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;

    println!("Initialized empty Git repository in .git/");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: git-rs <command>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "init" => cmd_init()?,
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            std::process::exit(1);
        }
    }
    Ok(())
}
```

**Rust concepts in play:**

- `fs::create_dir_all` — creates directories recursively; returns `Result<(), io::Error>`
- The `?` operator — if any directory creation fails, the error propagates to `main`
- `fs::write` — atomically writes a string/bytes to a file

**❓ Questions for Phase 1:**

1. What does `create_dir_all` do if the directory already exists? (Run `git-rs init` twice and observe)
2. Why does `HEAD` contain a reference to a branch name rather than directly to a commit hash?
3. What is the `Ok(())` at the end of `cmd_init`? What does `()` mean in Rust?

**✅ Verification:**

```bash
cargo build
./target/debug/git-rs init
ls -la .git/
cat .git/HEAD
# Should print: ref: refs/heads/main
git status  # The official git should recognize this as a valid (empty) repo
```

---

## Phase 2 — `hash-object -w <file>`

**Goal:** Read a file, format it as a blob, hash it, compress it, write it.

**Add `src/object.rs`:**

```rust
// src/object.rs

use sha1::{Sha1, Digest};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::io::Write;
use std::fs;
use std::path::PathBuf;

/// Returns the filesystem path for a given 40-char hex hash.
/// hash "da39a3ee5e6b4b0d3255bfef95601890afd80709"
///   → PathBuf(".git/objects/da/39a3ee5e6b4b0d3255bfef95601890afd80709")
pub fn object_path(hash: &str) -> PathBuf {
    // hash[0..2] = first two hex chars (directory name)
    // hash[2..] = remaining 38 chars (filename)
    PathBuf::from(format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]))
}

/// Given a type string ("blob", "tree", "commit") and raw content bytes,
/// constructs the full Git object bytes (header + content),
/// computes the SHA-1 hash, compresses, writes to .git/objects/,
/// and returns the 40-char hex hash.
pub fn write_object(
    kind: &str,
    content: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Build the header: "blob 12\0" or "tree 45\0" etc.
    //    The \0 is a null byte — byte value 0, the separator between header and content
    let header = format!("{} {}\0", kind, content.len());
    let header_bytes = header.as_bytes();

    // 2. Concatenate header + content into a single buffer
    //    This is the full object data that gets hashed and compressed
    let mut full_object: Vec<u8> = Vec::with_capacity(header_bytes.len() + content.len());
    full_object.extend_from_slice(header_bytes);  // Append header bytes
    full_object.extend_from_slice(content);       // Append content bytes

    // 3. Compute SHA-1 hash of the full object (header + content)
    //    IMPORTANT: Hash BEFORE compression. The hash is of the raw data.
    let mut hasher = Sha1::new();
    hasher.update(&full_object);           // Feed all bytes to the hasher
    let hash_bytes = hasher.finalize();    // Consume hasher, get 20-byte result

    // 4. Convert 20 raw bytes → 40 hex chars
    //    Each byte becomes 2 hex digits: 0xda → "da"
    let hash_hex: String = hash_bytes
        .iter()
        .map(|b| format!("{:02x}", b))    // {:02x} = hex, lowercase, padded to 2 chars
        .collect();

    // 5. Compress the full object using Zlib
    //    ZlibEncoder wraps a Vec<u8> as its output target
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&full_object)?;      // Feed data to compressor
    let compressed = encoder.finish()?;   // Flush compressor, get compressed bytes

    // 6. Determine the output path and create its directory
    let path = object_path(&hash_hex);
    let dir = path.parent().unwrap();       // ".git/objects/da" part
    fs::create_dir_all(dir)?;              // Create the 2-char directory if needed

    // 7. Write compressed bytes to disk
    fs::write(&path, &compressed)?;

    // 8. Return the hash so the caller can use it
    Ok(hash_hex)
}

/// Reads a stored Git object from .git/objects/ by its hash,
/// decompresses it, strips the header, and returns (kind, content).
pub fn read_object(
    hash: &str,
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    // 1. Locate the file
    let path = object_path(hash);

    // 2. Read the compressed bytes from disk
    let compressed = fs::read(&path)?;

    // 3. Decompress
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut raw: Vec<u8> = Vec::new();
    decoder.read_to_end(&mut raw)?;
    // raw now contains: "blob 12\0actual content bytes"

    // 4. Find the null byte that separates header from content
    //    iter().position() finds the first index satisfying the condition
    let null_pos = raw.iter().position(|&b| b == 0)
        .ok_or("No null byte in object")?;
    // ok_or() converts Option to Result — if None, return Err("No null byte...")

    // 5. Parse the header
    //    Header is everything before the null byte, as UTF-8 text
    let header = std::str::from_utf8(&raw[..null_pos])?;
    // header: "blob 12" or "tree 45" or "commit 200"

    // 6. Split header into type and size
    let mut parts = header.splitn(2, ' ');  // Split on first space only
    let kind = parts.next().ok_or("Missing type")?.to_string();
    // We parse the size but don't need to use it — the content slice IS the content
    let _size: usize = parts.next().ok_or("Missing size")?.parse()?;

    // 7. Content is everything after the null byte
    let content = raw[null_pos + 1..].to_vec();

    Ok((kind, content))
}
```

**Add the module to `main.rs` and the new command:**

```rust
// src/main.rs
mod object;  // Tell Rust to load src/object.rs

// ... (keep cmd_init from Phase 1)

fn cmd_hash_object(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read(path)?;           // Read file bytes
    let hash = object::write_object("blob", &content)?;  // Store as blob
    println!("{}", hash);                          // Print the 40-char hash
    Ok(())
}

fn cmd_cat_file(
    flag: &str,
    hash: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (kind, content) = object::read_object(hash)?;

    match flag {
        "-p" => {
            // Pretty-print: for blobs, content is raw bytes (probably UTF-8)
            print!("{}", String::from_utf8_lossy(&content));
        }
        "-t" => {
            // Print the type: "blob", "tree", or "commit"
            println!("{}", kind);
        }
        "-s" => {
            // Print the size in bytes
            println!("{}", content.len());
        }
        _ => eprintln!("Unknown flag: {}", flag),
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: git-rs <command> [args]");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "init" => cmd_init()?,
        "hash-object" => {
            // Expect: git-rs hash-object -w <file>
            if args.len() < 4 || args[2] != "-w" {
                eprintln!("Usage: git-rs hash-object -w <file>");
                std::process::exit(1);
            }
            cmd_hash_object(&args[3])?;
        }
        "cat-file" => {
            // Expect: git-rs cat-file <flag> <hash>
            if args.len() < 4 {
                eprintln!("Usage: git-rs cat-file <-p|-t|-s> <hash>");
                std::process::exit(1);
            }
            cmd_cat_file(&args[2], &args[3])?;
        }
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            std::process::exit(1);
        }
    }
    Ok(())
}
```

**Rust concepts in play:**

- `Vec::with_capacity(n)` — pre-allocates heap space, avoiding repeated reallocations as you extend it
- `extend_from_slice` — appends a byte slice without consuming the source
- `iter().position(|&b| b == 0)` — finds the index of the first null byte using a closure
- `splitn(2, ' ')` — splits a string but stops after the first 2 parts
- `ok_or()` — converts `Option<T>` to `Result<T, E>` by providing an error value for `None`
- `String::from_utf8_lossy()` — converts bytes to a string, replacing invalid UTF-8 with `?`

**❓ Questions for Phase 2:**

1. In `write_object`, why is `full_object` declared as `Vec<u8>` with `with_capacity`? What would happen if you used `Vec::new()` instead? (Hint: think about reallocations)
2. The `?` in `encoder.finish()?` — what two things does `finish()` do that both can fail?
3. In `read_object`, after finding the null byte, the content is `raw[null_pos + 1..]`. Why `null_pos + 1` and not `null_pos`?
4. `String::from_utf8_lossy(&content)` vs `String::from_utf8(content)` — what is the difference? When would you use each?
5. Why do you hash the full object BEFORE compression, rather than hashing the compressed bytes?

**✅ Verification:**

```bash
# Initialize a test repo
cd /tmp && mkdir test-git-rs && cd test-git-rs
/path/to/git-rs init

# Create a test file
echo "hello world" > hello.txt

# Hash it with YOUR binary
/path/to/git-rs hash-object -w hello.txt
# → prints something like: 8ab686eafeb1f44702738c8b0f24f2567c36da6d

# Read it back with official Git
git cat-file -p 8ab686eafeb1f44702738c8b0f24f2567c36da6d
# → "hello world"   ← if this works, your blob format is correct

# Verify type
git cat-file -t 8ab686eafeb1f44702738c8b0f24f2567c36da6d
# → "blob"

# Read with YOUR binary
/path/to/git-rs cat-file -p 8ab686eafeb1f44702738c8b0f24f2567c36da6d
# → "hello world"
```

---

## Phase 3 — `write-tree`

**Goal:** Snapshot the current directory as a tree object.

**What a tree object actually contains (binary, not text):**

```
For a directory containing:
  hello.txt  (hash: 8ab686...)
  world.txt  (hash: cc628c...)

The tree content bytes are:
  "100644 hello.txt\0" + [0x8a, 0xb6, 0x86, ... 20 raw bytes of hash]
  "100644 world.txt\0" + [0xcc, 0x62, 0x8c, ... 20 raw bytes of hash]

concatenated into a single flat byte sequence, then wrapped with:
  "tree <total_byte_length>\0" + <all_entries_bytes>
```

**Add `src/tree.rs`:**

```rust
// src/tree.rs

use crate::object;  // Access write_object and read_object from object.rs
use std::fs;
use std::path::Path;

pub struct TreeEntry {
    pub mode: String,     // "100644", "100755", "040000"
    pub name: String,     // filename or directory name
    pub hash: String,     // 40-char hex hash
}

/// Recursively builds tree objects for a directory and all subdirectories.
/// Returns the 40-char hex hash of the root tree object.
pub fn write_tree(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut entries: Vec<TreeEntry> = Vec::new();

    // Iterate over entries in the directory
    let mut dir_entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())   // Ignore unreadable entries
        .collect();

    // Sort entries by name — Git requires alphabetical order in tree objects
    dir_entries.sort_by_key(|e| e.file_name());

    for entry in dir_entries {
        let path = entry.path();
        let name = entry.file_name().into_string().unwrap();

        // Skip the .git directory itself — we don't want to store the store
        if name == ".git" {
            continue;
        }

        if path.is_file() {
            // For files: read content, store as blob, record the entry
            let content = fs::read(&path)?;
            let hash = object::write_object("blob", &content)?;
            entries.push(TreeEntry {
                mode: "100644".to_string(),
                name,
                hash,
            });
        } else if path.is_dir() {
            // For subdirectories: recurse to build a sub-tree
            let hash = write_tree(&path)?;   // Recursive call
            entries.push(TreeEntry {
                mode: "040000".to_string(),
                name,
                hash,
            });
        }
    }

    // Serialize entries into the binary tree format
    let content = serialize_tree_entries(&entries)?;

    // Store as a "tree" object
    object::write_object("tree", &content)
}

/// Converts a list of TreeEntry into the binary format Git expects.
fn serialize_tree_entries(
    entries: &[TreeEntry],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut content: Vec<u8> = Vec::new();

    for entry in entries {
        // Write mode (ASCII text): "100644" or "040000"
        content.extend_from_slice(entry.mode.as_bytes());

        // Write a space separator (ASCII 0x20)
        content.push(b' ');

        // Write filename (ASCII text): "hello.txt"
        content.extend_from_slice(entry.name.as_bytes());

        // Write null byte separator
        content.push(0u8);

        // Write the 20 RAW BYTES of the hash (NOT the 40-char hex string)
        // We must convert the hex string back to raw bytes here
        let hash_bytes = hex_to_bytes(&entry.hash)?;
        content.extend_from_slice(&hash_bytes);
    }

    Ok(content)
}

/// Converts a 40-char hex string to 20 raw bytes.
/// "da39a3" → [0xda, 0x39, 0xa3, ...]
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Step 2 chars at a time
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| e.into())  // Convert parse error to Box<dyn Error>
        })
        .collect()  // Collect Result<Vec<u8>, _> — if any conversion fails, returns Err
}

/// Parses a tree object's content bytes back into TreeEntry structs.
/// Useful for reading existing trees (cat-file, ls-tree).
pub fn parse_tree(content: &[u8]) -> Vec<TreeEntry> {
    let mut entries = Vec::new();
    let mut i = 0;

    while i < content.len() {
        // Read mode: bytes until the space
        let space_pos = content[i..].iter().position(|&b| b == b' ').unwrap() + i;
        let mode = String::from_utf8_lossy(&content[i..space_pos]).to_string();
        i = space_pos + 1;  // Skip past the space

        // Read filename: bytes until the null byte
        let null_pos = content[i..].iter().position(|&b| b == 0).unwrap() + i;
        let name = String::from_utf8_lossy(&content[i..null_pos]).to_string();
        i = null_pos + 1;  // Skip past the null byte

        // Read 20 raw hash bytes and convert to hex string
        let hash_bytes = &content[i..i + 20];
        let hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        i += 20;

        entries.push(TreeEntry { mode, name, hash });
    }
    entries
}
```

**Add to `main.rs`:**

```rust
mod tree;  // Add this at the top with mod object;

// In the match block:
"write-tree" => {
    let hash = tree::write_tree(std::path::Path::new("."))?;
    println!("{}", hash);
}
"ls-tree" => {
    if args.len() < 3 {
        eprintln!("Usage: git-rs ls-tree <hash>");
        std::process::exit(1);
    }
    let (_kind, content) = object::read_object(&args[2])?;
    let entries = tree::parse_tree(&content);
    for e in entries {
        println!("{} {} {}\t{}", e.mode,
            if e.mode == "040000" { "tree" } else { "blob" },
            e.hash, e.name);
    }
}
```

**Rust concepts in play:**

- `fs::read_dir()` — returns an iterator over directory entries, each wrapped in `Result`
- `filter_map(|e| e.ok())` — converts each `Result<DirEntry, Error>` to `Option<DirEntry>`, discarding errors and unwrapping successes
- `sort_by_key()` — sorts in-place using a key extraction closure
- `into_string().unwrap()` — converts `OsString` (OS-native string) to Rust `String`; can fail on non-UTF8 filenames
- `(0..hex.len()).step_by(2)` — iterator that yields 0, 2, 4, 6... — processes two hex chars at a time
- `.collect::<Result<Vec<u8>, _>>()` — collects an iterator of `Result`s into a single `Result<Vec>`: if any element is `Err`, the whole collection is `Err`

**❓ Questions for Phase 3:**

1. Why must tree entries be sorted alphabetically? (Run `git ls-tree` on a real repo — what order does it use?)
2. Why does the tree format use raw 20-byte hashes instead of the 40-char hex string? Calculate: what is the size difference for a tree with 100 entries?
3. In `parse_tree`, you read exactly 20 bytes for the hash. What would happen if you were parsing a corrupt tree object where one hash was only 19 bytes?
4. The `collect::<Result<Vec<u8>, _>>()` pattern in `hex_to_bytes` — explain in your own words what `collect()` does when the iterator produces `Result` values.

**✅ Verification:**

```bash
/path/to/git-rs write-tree
# → prints a tree hash like: 4b825dc642cb6eb9a060e54bf8d69288fbee4904

# Read the tree with official Git
git cat-file -p 4b825dc642cb6eb9a060e54bf8d69288fbee4904
# → lists the files and their hashes

# Verify with your ls-tree
/path/to/git-rs ls-tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904
```

---

## Phase 4 — `commit-tree`

**Goal:** Create a commit object that references a tree.

**Add `src/commit.rs`:**

```rust
// src/commit.rs

use crate::object;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CommitData {
    pub tree: String,         // 40-char hex hash of the tree
    pub parent: Option<String>, // 40-char hex hash of parent commit, if any
    pub author_name: String,
    pub author_email: String,
    pub message: String,
    pub timestamp: u64,       // Unix timestamp
    pub timezone: String,     // e.g., "+0200"
}

impl CommitData {
    pub fn new(tree: String, parent: Option<String>, message: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        CommitData {
            tree,
            parent,
            author_name: "Anonymous".to_string(),
            author_email: "anon@example.com".to_string(),
            message,
            timestamp,
            timezone: "+0000".to_string(),
        }
    }

    /// Serialize the commit to the bytes that go inside the "commit <size>\0" wrapper.
    pub fn serialize(&self) -> Vec<u8> {
        let mut lines = Vec::new();

        lines.push(format!("tree {}", self.tree));

        if let Some(parent) = &self.parent {
            lines.push(format!("parent {}", parent));
        }

        let author_line = format!(
            "author {} <{}> {} {}",
            self.author_name, self.author_email, self.timestamp, self.timezone
        );
        lines.push(author_line.clone());
        lines.push(author_line.replace("author", "committer"));

        // Empty line separates headers from message
        lines.push(String::new());
        lines.push(self.message.clone());

        lines.join("\n").into_bytes()
    }
}

/// Creates and stores a commit object. Returns its hash.
pub fn create_commit(
    tree_hash: &str,
    parent_hash: Option<&str>,
    message: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let data = CommitData::new(
        tree_hash.to_string(),
        parent_hash.map(|s| s.to_string()),
        message.to_string(),
    );

    let content = data.serialize();
    object::write_object("commit", &content)
}

/// Parses a commit object's content bytes into a CommitData struct.
pub fn parse_commit(content: &[u8]) -> Result<CommitData, Box<dyn std::error::Error>> {
    let text = std::str::from_utf8(content)?;
    let mut lines = text.lines();

    let mut tree = String::new();
    let mut parent = None;
    let mut author_name = String::new();
    let mut author_email = String::new();
    let mut timestamp = 0u64;
    let mut timezone = String::new();
    let mut message = String::new();
    let mut in_message = false;

    for line in &mut lines {
        if in_message {
            message.push_str(line);
            message.push('\n');
            continue;
        }
        if line.is_empty() {
            in_message = true;
            continue;
        }
        if let Some(hash) = line.strip_prefix("tree ") {
            tree = hash.to_string();
        } else if let Some(hash) = line.strip_prefix("parent ") {
            parent = Some(hash.to_string());
        } else if let Some(author) = line.strip_prefix("author ") {
            let parts: Vec<&str> = author.rsplitn(3, ' ').collect();
            if parts.len() >= 3 {
                timezone = parts[0].to_string();
                timestamp = parts[1].parse().unwrap_or(0);
                let name_email = parts[2];
                if let (Some(lt), Some(gt)) = (name_email.find('<'), name_email.find('>')) {
                    author_email = name_email[lt+1..gt].to_string();
                    author_name = name_email[..lt].trim().to_string();
                }
            }
        }
    }

    Ok(CommitData { tree, parent, author_name, author_email,
                    message, timestamp, timezone })
}
```

**Add to `main.rs`:**

```rust
mod commit;

"commit-tree" => {
    if args.len() < 5 {
        eprintln!("Usage: git-rs commit-tree <tree> [-p <parent>] -m <msg>");
        std::process::exit(1);
    }
    let tree_hash = &args[2];
    let mut parent: Option<&str> = None;
    let mut message = "";
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "-p" => { parent = Some(&args[i + 1]); i += 2; }
            "-m" => { message = &args[i + 1]; i += 2; }
            _ => { i += 1; }
        }
    }
    let hash = commit::create_commit(tree_hash, parent, message)?;
    println!("{}", hash);
}
```

**❓ Questions for Phase 4:**

1. The `serialize()` method uses `lines.join("\n").into_bytes()`. What does `into_bytes()` do? What does it consume?
2. Why does a first commit have no `parent` field? What does `Option<String>` allow you to express that a plain `String` cannot?
3. `parent_hash.map(|s| s.to_string())` — `parent_hash` is `Option<&str>`. What does calling `.map()` on an `Option` do?

**✅ Verification:**

```bash
TREE=$(/path/to/git-rs write-tree)
/path/to/git-rs commit-tree $TREE -m "First commit"
git cat-file -p <commit-hash>
# → shows tree, author, committer, and message
```

---

## Phase 5 — Refs and the `commit` High-Level Command

**Add `src/refs.rs`:**

```rust
// src/refs.rs

use std::fs;
use std::path::PathBuf;

pub fn read_head() -> Result<String, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(".git/HEAD")?;
    let contents = contents.trim();
    if let Some(branch) = contents.strip_prefix("ref: ") {
        Ok(branch.to_string())
    } else {
        Ok(contents.to_string())
    }
}

pub fn read_ref(ref_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let path = PathBuf::from(".git").join(ref_name);
    if !path.exists() { return Ok(None); }
    let hash = fs::read_to_string(&path)?.trim().to_string();
    Ok(Some(hash))
}

pub fn write_ref(ref_name: &str, hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(".git").join(ref_name);
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    fs::write(&path, format!("{}\n", hash))?;
    Ok(())
}

pub fn current_commit() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let head = read_head()?;
    if head.starts_with("refs/") { read_ref(&head) }
    else { Ok(Some(head)) }
}

pub fn update_current_ref(hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let head = read_head()?;
    if head.starts_with("refs/") { write_ref(&head, hash)?; }
    Ok(())
}
```

**Add the `commit` command to `main.rs`:**

```rust
mod refs;

"commit" => {
    let msg_idx = args.iter().position(|a| a == "-m")
        .ok_or("Expected -m flag")?;
    let message = args.get(msg_idx + 1).ok_or("Expected message after -m")?;
    let tree_hash = tree::write_tree(std::path::Path::new("."))?;
    let parent = refs::current_commit()?;
    let commit_hash = commit::create_commit(
        &tree_hash, parent.as_deref(), message)?;
    refs::update_current_ref(&commit_hash)?;
    println!("[{}] {}", &commit_hash[..7], message);
}
```

**❓ Questions for Phase 5:**

1. `parent.as_deref()` converts `Option<String>` to `Option<&str>`. Why is `&str` preferred over `String` in function parameters?
2. `args.iter().position(|a| a == "-m")` returns what type? What does `.ok_or()` do with it?
3. What is "detached HEAD" state? Why might you end up in it?

**✅ Verification:**

```bash
echo "change 1" >> hello.txt
/path/to/git-rs commit -m "Second commit"
echo "change 2" >> hello.txt
/path/to/git-rs commit -m "Third commit"
git log --oneline
# → Should show your commits in order
```

---

<a name="part-6"></a>

# Part 6 - The LLM Wiki Extension

---

## 6.1 The Core Problem (Why You Can't Just Point an LLM at `.git/`)

If you run `cat .git/objects/da/39a3ee5e...`, you see binary garbage. Two reasons:

1. The object is Zlib-compressed. The decompressed content is human-readable text or binary — either way, raw bytes are not LLM input.
2. Even decompressed, a blob has a `blob 12\0` header. A tree object's content is entirely binary (raw hash bytes, no text at all).

**The LLM needs clean text.** Your Rust CLI is the only system that knows how to reconstruct clean text from the object database. This makes it the mandatory translation layer — the "syscall interface" between the binary store and the semantic layer.

---

## 6.2 The Architecture

```markdown
  YOUR WORKING DIRECTORY
  ───────────────────────────────────────────────────────
  src/main.rs, src/object.rs         ← real source files
  .git/objects/...                   ← your compressed DAG

           │
           ▼
  ┌───────────────────────────────────────────────────┐
  │  git-rs export-snapshot                           │
  │                                                   │
  │  HEAD → branch → commit hash                      │
  │  commit → tree hash                               │
  │  tree → recursive walk → blobs                    │
  │  each blob → decompress + strip header            │
  │  output: structured JSON                          │
  └──────────────────────┬────────────────────────────┘
                         │
                         ▼
  snapshot.json:
  {
    "commit": "da39a3...",
    "message": "Implement blob storage",
    "files": [
      { "path": "src/main.rs", "content": "fn main() {...}" },
      { "path": "src/object.rs", "content": "pub fn write_object..." }
    ]
  }
                         │
                         ▼
  ┌────────────────────────────────────────────────────┐
  │  Local LLM (Ollama / Mistral / GLM)                │
  │                                                    │
  │  Receives structured snapshot                      │
  │  Generates / updates wiki pages:                   │
  │    - What this module does                         │
  │    - Why the architecture changed since last commit│
  │    - Cross-references between files                │
  │    - Open questions and TODOs                      │
  └────────────────────────────────────────────────────┘
```

---

## 6.3 The `export-snapshot` Command

```rust
// Add to main.rs
"export-snapshot" => {
    let snapshot = export_snapshot()?;
    println!("{}", snapshot);
}

fn export_snapshot() -> Result<String, Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    let commit_hash = refs::current_commit()?.ok_or("No commits yet")?;
    let (_kind, commit_content) = object::read_object(&commit_hash)?;
    let commit_data = commit::parse_commit(&commit_content)?;

    let mut files: HashMap<String, String> = HashMap::new();
    walk_tree(&commit_data.tree, "", &mut files)?;

    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"commit\": \"{}\",\n", commit_hash));
    output.push_str(&format!("  \"message\": \"{}\",\n",
        commit_data.message.trim().replace('"', "\\\"")));
    output.push_str("  \"files\": [\n");

    let file_vec: Vec<(&String, &String)> = files.iter().collect();
    for (i, (path, content)) in file_vec.iter().enumerate() {
        let comma = if i < file_vec.len() - 1 { "," } else { "" };
        let escaped = content.replace('\\', "\\\\").replace('"', "\\\"")
                              .replace('\n', "\\n");
        output.push_str(&format!(
            "    {{\"path\": \"{}\", \"content\": \"{}\"}}{}\n",
            path, escaped, comma));
    }
    output.push_str("  ]\n}\n");
    Ok(output)
}

fn walk_tree(
    tree_hash: &str,
    prefix: &str,
    files: &mut std::collections::HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_kind, content) = object::read_object(tree_hash)?;
    let entries = tree::parse_tree(&content);

    for entry in entries {
        let full_path = if prefix.is_empty() {
            entry.name.clone()
        } else {
            format!("{}/{}", prefix, entry.name)
        };

        if entry.mode == "040000" {
            walk_tree(&entry.hash, &full_path, files)?;
        } else {
            let (_kind, blob_content) = object::read_object(&entry.hash)?;
            files.insert(full_path, String::from_utf8_lossy(&blob_content).to_string());
        }
    }
    Ok(())
}
```

---

## 6.4 The Post-Commit Hook

Wire the export + LLM invocation into a Git hook so it runs automatically on every commit:

```bash
#!/bin/bash
# .git/hooks/post-commit  (make executable: chmod +x .git/hooks/post-commit)

SNAPSHOT=$(/path/to/git-rs export-snapshot)
DATE=$(date +%Y-%m-%d-%H%M)

echo "$SNAPSHOT" | ollama run mistral \
  "You are a documentation agent. Given this JSON codebase snapshot, \
   write a concise Markdown wiki page: architecture summary, key functions, \
   data flow, and open questions. Output only Markdown, no preamble." \
  > docs/wiki/${DATE}.md

echo "Wiki updated: docs/wiki/${DATE}.md"
```

---

<a name="appendix-a"></a>

# Appendix A - Questions for Deep Retention

Answer these after your first read-through. Answer them again after Phase 5 is complete. The gap between the two attempts is a direct measure of what you actually learned.

**Rust:**

1. Explain the difference between `String`, `&String`, `&str`, `Vec<u8>`, and `&[u8]`. One concrete use case each.
2. When does Rust free a value? What is the rule?
3. Why can't you have `&mut T` and `&T` to the same data simultaneously? What problem does this prevent?
4. Write out what `let x = thing()?;` expands to as a full `match` expression.
5. Why does `Box<dyn Error>` exist? What breaks if every function names its exact error type?
6. Explain `collect::<Result<Vec<u8>, _>>()` step by step.
7. What is `iter()` vs `into_iter()` vs `iter_mut()`? What is the ownership difference?
8. What does `Vec::with_capacity(n)` do differently than `Vec::new()`? Why does it matter for performance?

**Git Internals:**
9. What is content-addressable storage? Name two SHA-1 properties that make it suitable.
10. Describe the byte structure of a Git blob for the content `"hi\n"`. Write the exact bytes.
11. Why does a tree object store raw 20-byte hashes instead of 40-char hex strings? Calculate the space difference for 100 entries.
12. If you edit one line in one file and make a new commit, which objects are newly created? Which are reused?
13. What is the DAG? Why is it acyclic? What would a cycle mean?
14. How does Git detect retroactive tampering?
15. At what point in the pipeline is Zlib compression applied? Why is it applied after hashing, not before?

**Architecture:**
16. Draw the complete data flow for `git-rs commit -m "message"`. Include every function call and every file touched.
17. Why is `export-snapshot` architecturally necessary for LLM integration? Why can't you skip it?
18. How would you implement `git diff` using the objects you already know? Which two objects would you compare?

---

<a name="appendix-b"></a>

# Appendix B - Verification Tests Per Phase

| Phase | Your Command | Expected Output | Verified With |
| ------- | ------------- | ----------------- | --------------- |
| 1 | `git-rs init && cat .git/HEAD` | `ref: refs/heads/main` | your binary |
| 1 | `git-rs init && git status` | `On branch main` | official git |
| 2 | `echo "test" > f.txt && git-rs hash-object -w f.txt` | 40-char hex | your binary |
| 2 | `git cat-file -t <hash>` | `blob` | official git |
| 2 | `git cat-file -p <hash>` | `test` | official git |
| 2 | `git-rs cat-file -p <hash>` | `test` | your binary |
| 3 | `git-rs write-tree` | 40-char hex | your binary |
| 3 | `git cat-file -p <tree-hash>` | file listing | official git |
| 3 | `git-rs ls-tree <tree-hash>` | file listing | your binary |
| 4 | `git-rs commit-tree <tree> -m "msg"` | 40-char hex | your binary |
| 4 | `git cat-file -p <commit-hash>` | tree/author/message | official git |
| 5 | `git-rs commit -m "first"` | `[abc1234] first` | your binary |
| 5 | `git log --oneline` | commit hash + message | official git |
| 6 | `git-rs export-snapshot` | valid JSON with file contents | your binary |

---

# The North Star

At some point during Phase 2, you will write the hash, check it with `git cat-file`, and it will work. The official Git binary will read an object you created from first principles in Rust, and it will return the exact content you put in.

That moment is the point of the whole exercise. Not because it is impressive. Because in that moment you will understand — not abstractly, but concretely — that `git` is not magic. It is a well-specified byte format and a content-addressable filesystem. Everything else is built on top of those two ideas.

The rest of the document is just details.

---

*Document version 1.0 — May 2026*
*Target reader: Fady, Phase 22.1*
