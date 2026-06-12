# Lessons Learned: Phase 3

> Concrete Rust and systems programming lessons from building `write-tree`. Each lesson is tied to a specific moment in the implementation, not stated in the abstract.

---

## Rust Lessons

### 1. The call stack is already a data structure

The initial plan for traversing the directory tree involved managing a manual stack: push items, track which parent you came from, detect when to assemble a tree.

Recursion renders this unnecessary. When a function calls itself, the OS call stack stores the current function's local variables and the point to return to. When the recursive call returns, execution resumes at the call site with everything intact — including the partial list of entries being built for the parent directory. The state tracking that would require dozens of lines of manual management happens automatically.

This is not a Rust-specific insight, but Rust's type system makes it concrete: the `Vec<TreeEntry>` declared at the start of each function invocation is owned by that invocation. It is not shared with the recursive calls. Each level of the directory tree has its own list. There is no aliasing, no mutation-at-a-distance, no need to clear and reuse.

### 2. `OsString` is not `String`: the OS boundary exists for a reason

`fs::read_dir()` yields entries whose `file_name()` method returns `OsString`, not `String`. These are different types in Rust, and they are different for a specific reason: operating systems do not guarantee that file paths are valid UTF-8. A path on Linux is a sequence of bytes where the only reserved byte is `0x00`. Most paths are UTF-8, but not all.

Rust's `String` type guarantees UTF-8. `OsString` does not. The conversion between them is handled explicitly via `.to_string_lossy()`, which converts valid UTF-8 sequences normally and replaces invalid sequences with the Unicode replacement character rather than panicking.

In this project, `.to_string_lossy().into_owned()` appears in every place a filename is extracted from a `DirEntry`. It works correctly for the practical case (all filenames in the test repository are ASCII). The lesson is that the type system exposes a real boundary — the gap between what the OS guarantees and what your code assumes — and forces you to handle it explicitly.

### 3. `Vec<u8>` is the serialization target

Coming from a language with objects and serialization frameworks, the instinct is to build a structured representation (a `Vec<TreeEntry>`) and pass it to some function that knows how to convert it to disk format.

Git does not know what a `Vec<TreeEntry>` is. Git reads bytes. The structured in-memory representation is for the programmer. Before anything touches disk, the structure must be flattened into a continuous sequence of bytes that matches the specification exactly.

`Vec<u8>` is that sequence. Two methods do all the work:

- `.push(byte)` appends a single byte — used for the space delimiter and the null terminator
- `.extend_from_slice(&bytes)` appends a slice of bytes — used for the mode string, the filename, and the 20 raw hash bytes

Both the mode string and the filename are converted to byte slices via `.as_bytes()`. The 20 raw hash bytes require a separate conversion from the 40-character hex string, since the hex string is a human-readable encoding of the raw bytes, not the raw bytes themselves.

There is no abstraction layer between the programmer and the bytes. The programmer decides what goes where.

### 4. The `?` operator is a control flow statement

`?` appears after nearly every fallible operation in this codebase: `fs::read(path)?`, `write_object("blob", &content)?`, `hex_to_bytes(&entry.hash)?`. In every case, the meaning is the same: if this returns `Err`, return that `Err` from the current function immediately.

What makes this worth noting is the interaction with recursion. When `write_tree` calls itself recursively and uses `?` on the result, a failure deep inside a nested subdirectory propagates outward through every level of the call stack without any explicit error handling at the intermediate levels. One `Err` at the leaf level unwinds the entire recursion and reports the error to the top-level caller. This is composable error propagation: each function either succeeds and returns its value, or fails and lets the caller decide what to do.

### 5. Cloning during sort: a tradeoff to revisit

The sort line is `.sort_by_key(|k| k.name.clone())`. The `.clone()` creates a new `String` for every comparison the sort algorithm makes. For small directories this is fine. For directories with thousands of entries, it means thousands of heap allocations during a sort.

The idiomatic fix is to sort by a reference: `.sort_by(|a, b| a.name.cmp(&b.name))`. This compares the names in place without allocating copies.

The current implementation uses `.clone()` because it was the first thing that satisfied the borrow checker. The compiler was telling a real truth — you cannot hand out mutable references to the vector (for sorting) while also holding immutable references into its elements. The `.clone()` satisfies this by avoiding the references entirely.

Understanding why `.sort_by()` with references also satisfies the borrow checker — the comparator borrows elements only during the comparison and releases those borrows before the next swap — is the next step. That reasoning is worth working through before making the change.

---

## Systems Programming Lessons

### 6. Specification errors are invisible to the compiler

The two most damaging bugs in this implementation — the wrong mode string (`"104000"` instead of `"100644"`) and the case-insensitive sort — both compiled cleanly and ran without panicking. The program produced output that looked correct: a 40-character hex string. The error was only visible when that hash was passed to the official Git binary.

This is the category of bug that matters most in systems programming. The compiler checks types, ownership, and lifetimes. It does not check whether `"104000"` is a valid Git mode value. It does not know that Git sorts by raw ASCII order, not Unicode collation. That knowledge lives in the specification, and checking the implementation against the specification is the programmer's job.

The practical implication: for any binary format you implement, the verification step is not "does it compile" or "does it run". It is "does the official implementation agree with mine". In this project, `git cat-file -p <hash>` is the specification made executable.

### 7. Skipping the staging area is a deliberate shortcut

In official Git, `git add` is what creates blob objects and records their hashes in the index. `git write-tree` reads from the index — it never touches the working directory files directly.

`git-rs write-tree` does both: it reads files from the working directory and creates blobs on-the-fly. This works correctly and produces valid output, but it means `write-tree` will create a new blob object every time it runs, even if the file has not changed. In official Git, an unchanged file added with `git add` points to the same blob object across every `write-tree` call.

This shortcut is appropriate for Phase 3. Implementing the index (a binary file format with its own spec) is a separate project. The shortcut will need to be unwound when `git-rs commit` is implemented properly, because `commit` needs to distinguish between staged changes and unstaged changes — a distinction that requires the index.

### 8. The Directed Acyclic Graph is enforced by the algorithm, not by a constraint

There is no code in `write-tree` that checks "is this a cycle?" or "have I visited this directory before?" (beyond the `.git` guard). The DAG property is enforced by the algorithm's structure: post-order traversal means a node's hash is computed from the hashes of its children, which means a node cannot contain its own hash as a child — there is no mechanism by which a cycle could form. The hash of a directory cannot depend on itself because the hash does not exist until after all its children have been hashed.

This is a broader design principle: constraints that can be enforced structurally are more reliable than constraints enforced by runtime checks. The traversal order makes cycles impossible, not a check that looks for them.
