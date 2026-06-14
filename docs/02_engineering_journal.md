# Engineering Journal: Building `write-tree`

> A record of how Phase 3 was built — the specific decisions made, the specific bugs introduced, and the reasoning that resolved them. Documented for portfolio and future reference.
>
> **For Phase 4 (commit objects and DAG), see:**
>
> - [04_commit_object_and_commit_tree.md](04_commit_object_and_commit_tree.md)
> - [05_dag_and_commit_serialization.md](05_dag_and_commit_serialization.md)

---

## The Constraint

Before any code was written, a constraint was established: no generated implementations. The goal was not to produce working code as fast as possible. The goal was to own the logic and the data flow, and use the Rust compiler as a strict reviewer that would refuse to compile anything logically inconsistent.

This constraint changes the nature of the work. Every design decision had to be reasoned through in plain language before touching the keyboard. Every compiler error became a question: what did I misunderstand?

The result took more iterations than generated code would have. It also produced a more durable understanding of what `write-tree` actually does.

---

## Iteration 0: Designing the Algorithm Before Writing Code

The first question was architectural: what traversal order is required?

A tree object's hash is derived from the hashes of its children. To write the parent tree, you need the children's hashes first. To get a subdirectory's hash, you must have already hashed everything inside that subdirectory.

This dependency chain — children must be resolved before parents — has a name: **post-order depth-first traversal**. It is the only mathematically valid order for this problem. Any attempt to process the root first and work downward fails because the root's content depends on data that does not exist yet.

The initial plan was to implement this with a manual stack: push files onto a list, detect when you return to a parent, then hash the accumulated entries. This is a valid approach, but it requires tracking state manually — which directory you came from, which entries belong to which scope.

The pivot came from recognizing that directory structures are already trees, and that a function calling itself on subdirectories (recursion) lets the OS call stack handle all of that state automatically. When the recursive call returns, you are back in the parent directory with the child's hash in hand. No manual state tracking needed.

The algorithm in plain language:

```Psuedo Code
function build_tree(directory):
  create empty list of entries

  for each item in directory:
    if item name is ".git": skip

    if item is a file:
      read the file's bytes
      call write_object("blob", bytes) → get the blob hash
      add TreeEntry {mode: "100644", name, hash} to list

    if item is a directory:
      call build_tree(item) → get the subtree hash
      add TreeEntry {mode: "040000", name, hash} to list

  sort list by name, raw ASCII order

  serialize list to binary bytes:
    for each entry: mode + space + name + null byte + 20 raw hash bytes

  call write_object("tree", binary bytes) → get tree hash
  return tree hash
```

This design was locked in before the editor was opened.

---

## Iteration 1: First Skeleton

The first working draft had the right recursive structure. The function called itself on subdirectories and printed what it found.

Three bugs were present, none of which the compiler catches because they are logic errors, not type errors:

**Bug 1: Collecting filenames instead of entries.**
The collection was typed as `Vec<String>` and was storing only the filenames. At the end of the function there was no way to reconstruct the mode or the hash for any entry — that information was never captured.

The fix was to use the `TreeEntry` struct that already existed in `tree.rs`. The collection became `Vec<TreeEntry>`, and each entry stored all three fields at the point of creation.

**Bug 2: Sorting before the loop.**
The `sort_by_key` call appeared two lines after `Vec::new()`, before the loop had run. Sorting an empty vector does nothing, and then the loop populated it in whatever order the filesystem returned entries. By the end of the function, the vector was unsorted.

The sort must happen after the loop completes and the vector is fully populated. Moved to after the closing brace of the `for` loop.

**Bug 3: The `.git` check ran after the push.**
The guard against the `.git` directory appeared after the entry had already been pushed into the collection. The directory was excluded from traversal but its name was still recorded.

The check must run at the very top of the loop body, before any other operation. If the name is `.git`, call `continue` immediately.

---

## Iteration 2: Wiring the Actual Work

With the collection type and control flow fixed, the next task was replacing the `println!` calls in the file and directory branches with actual work.

**The file branch.** Reading the file with `fs::read()` and calling `write_object("blob", &content)` produces the hash. This hash, combined with the filename and mode `"100644"`, becomes a `TreeEntry` pushed onto the list.

One thing to note: this implementation writes blobs to the database during `write-tree`. A real Git separates this into `git add` (which writes blobs and updates an index file) and `git write-tree` (which reads from the index). Because `git-rs` has no staging area yet, `write-tree` takes on both responsibilities. The blob objects are created on-the-fly from the working directory.

**The directory branch.** The recursive call `write_tree(&entry.path())?` returns a `Result<String, ...>`. The `String` is the subtree's hash. This is the return value that must be captured and used to build the parent's `TreeEntry`. The early versions of this branch called the function and discarded the return value — the call happened but the hash was thrown away.

The fix: `let hashed_object = write_tree(&entry.path())?;` and then use `hashed_object` as the hash in the `TreeEntry`.

---

## Iteration 3: Two Bugs That Pass the Compiler

At this point the code compiled cleanly and ran without panicking. It produced a hash. The hash was wrong.

Running `git cat-file -p <hash>` with the wrong hash produces `fatal: Not a valid object name`. Finding the bugs required reading the code against the exact byte format specification, not relying on the compiler.

**Bug 1: Wrong file mode.**
The file mode was hardcoded as `"104000"`. Git does not recognize this value. The standard mode for a regular non-executable file is `"100644"`. This is a typo — transposed digits — and the compiler has no way to detect it because both are valid strings.

The consequence: the tree's binary payload contained `104000` in the mode field, which produces a different byte sequence than `100644`, which produces a different SHA-1 hash than official Git would compute for the same directory.

Fixed by correcting the string to `"100644"`.

**Bug 2: Case-insensitive sort.**
The sort used `.sort_by_key(|k| k.name.to_lowercase())`. This converts every filename to lowercase before comparing, which means `README.md` and `readme.md` would be treated as identical, and `Makefile` would sort after `build.sh` instead of before it.

Git sorts by raw ASCII byte values. In ASCII, uppercase letters (65–90) come before lowercase letters (97–122). `Makefile` (M = 77) sorts before `build.sh` (b = 98).

Forcing lowercase changes the sort key and therefore the sort order, which changes the binary payload, which changes the hash.

Fixed by sorting on the raw name: `.sort_by_key(|k| k.name.clone())`.

---

## Iteration 4: The Binary Serialization

With the collection correct and sorted, the final step was replacing `Ok("".to_string())` with the actual tree serialization.

The tree's binary payload is all entries concatenated. For each entry, the exact sequence is:

```Text
mode.as_bytes()     → the ASCII mode string as bytes
b' '                → one space byte
name.as_bytes()     → the filename as bytes
0x00                → one null byte
hex_to_bytes(&hash) → 20 raw bytes converted from the 40-char hex hash
```

A `Vec<u8>` is built by appending each of these in order, repeated for every entry. Then `write_object("tree", &formatted_tree_entries)` wraps it in the tree header, hashes it, compresses it, and writes it to `.git/objects/`. Its return value — the tree hash — is what the function returns to its caller.

The `hex_to_bytes` function was already present in `tree.rs`. It parses the 40-character hex string two characters at a time and converts each pair to the raw byte value. Using it here was the only way to produce the 20 raw bytes that Git's tree format requires.

---

## Iteration 5: Wiring to main.rs

The function was complete but not connected to anything. Two bugs existed in `main.rs`:

**Bug 1: Wrong path argument.**
The `write-tree` match arm called `cmd_write_tree(Path::new(&args[0]))`. The comment at the top of `main.rs` explicitly documented that `args[0]` is the path to the compiled binary itself (`target/debug/git-rs`). Passing this as the directory to scan would attempt to recurse into the build artifacts directory, not the working repository.

The correct path for "the current directory where the user is running this command" is `"."`. Fixed to `Path::new(".")`.

**Bug 2: Hash returned but not printed.**
The returned hash was captured in a variable but never printed. The match arm returned `Ok(())` immediately after the function call without printing anything to stdout.

Fixed by adding `println!("{}", tree_hash)` before `Ok(())`.

---

## The Verification

After all five iterations, the first clean end-to-end test against the official Git binary:

```Text
$ cargo run -- write-tree
"./sample.txt"
"./folder1/new-sample.txt"
"./folder1/file1.txt"
4d69c9857fa055de1b36ee9372f2b2bb92c844d5

$ git cat-file -p 4d69c9857fa055de1b36ee9372f2b2bb92c844d5
040000 tree 0ec3ee1930d31225f69076ad08a436b3a01d9908    folder1
100644 blob 1c2957419bb90febfe0c82b09dcbbed46fab37f1    sample.txt
```

The official Git binary read the database without error. The directory structure was correct. The alphabetical ordering was correct (`folder1` before `sample.txt` — `'f'` comes before `'s'` in ASCII). The subtree hash `0ec3ee...` can itself be inspected:

```Text
$ git cat-file -p 0ec3ee1930d31225f69076ad08a436b3a01d9908
100644 blob ...   file1.txt
100644 blob ...   new-sample.txt
```

Again correct. `file1.txt` before `new-sample.txt` — `'f'` before `'n'` in ASCII.

The two implementations agreed on the bytes. Phase 3 done.

---

## What This Approach Produced

The finished implementation contains bugs the compiler could not catch — wrong mode strings, case-insensitive sorting, sorting an empty list — and bugs the compiler does catch (wrong types, discarded return values). Both categories required different skills to find and fix.

The compiler errors pointed to type mismatches and discarded results. A compiler error is unambiguous: the program cannot be built until it is resolved. These are easy to work with.

The logic errors — wrong mode, wrong sort order — required understanding the specification and checking the output against it. No tool catches these. They are caught by testing against the official binary and reading the byte format carefully.

Both categories of debugging were necessary. The constraint of owning the logic meant both categories were encountered and resolved from first principles rather than by reading someone else's working solution.
