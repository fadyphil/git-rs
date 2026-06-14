# Git Internals: What Git Actually Is

> This document covers the architecture and byte-level contracts that `write-tree` is built on. It is scoped to Phase 3 of the `git-rs` project.
>
> **For Phase 4 (commit objects, DAG, and parent references), see:**
>
> - [04_commit_object_and_commit_tree.md](04_commit_object_and_commit_tree.md) — Commit object format and DAG structure
> - [05_dag_and_commit_serialization.md](05_dag_and_commit_serialization.md) — DAG mathematics and commit serialization

---

## 1. The Core Idea: Content-Addressable Storage

Git does not track files. It tracks **content**.

When Git stores a file, it generates an ID for that file's content using SHA-1. That ID — not the filename, not the path — is the permanent address of the content. This is called content-addressable storage.

Two files with identical content, anywhere in the repository, map to one object on disk. The ID is derived entirely from the bytes inside the file, not where the file lives.

```Text
Working directory:            .git/objects/:
────────────────────          ──────────────────────────────────────────
src/auth/token.txt            1c2957419bb90febfe0c82b09dcbbed46fab37f1
docs/token_example.txt  →     1c2957419bb90febfe0c82b09dcbbed46fab37f1
                                            ↑
                              same 6 bytes inside both files = one object
```

Two SHA-1 properties make this work for a storage system:

1. **Determinism.** The same input always produces the same 40-character output. Hash the same bytes twice and you get the same hash twice.
2. **Avalanche effect.** Change a single byte in the input and the output changes completely — there is no partial correlation between similar inputs and their hashes. This is what makes tampering detectable: editing a stored object changes its hash, which breaks every reference pointing to it.

---

## 2. The Four Object Types

Everything in Git's database is one of four object types. Phase 3 touches two of them directly.

| Object | Content | Format |
| -------- | --------- | -------- |
| **blob** | Raw bytes of a file | Binary (the file's content) |
| **tree** | A directory snapshot | Binary (encoded list of entries) |
| **commit** | A snapshot + author + message | Plain text |
| **tag** | An annotated pointer | Plain text |

A blob is what `hash-object` creates. A tree is what `write-tree` creates. A tree's entries can point to blobs (files) or other trees (subdirectories).

---

## 3. Object Storage: The Universal Wrapper

Every object — regardless of type — is stored the same way on disk:

```Text
Step 1 — Build the full object in memory:
  "<type> <content_length_in_bytes>\0<content_bytes>"

Step 2 — Compute SHA-1 of the full object:
  → 20 raw bytes → hex-encoded → 40-character string
  → this is the object's permanent ID

Step 3 — Zlib-compress the full object

Step 4 — Write to .git/objects/<first 2 hex chars>/<remaining 38 hex chars>
```

The hash is always computed over the **full object** — header included. Changing the header changes the hash. This is why the `type` and `size` fields in the header are not decorative: they are part of the cryptographic identity of the object.

An example with a file containing `"hello\n"` (6 bytes):

```Text
Content bytes:   68 65 6c 6c 6f 0a          (h e l l o LF)
Header:          "blob 6\0"
Header bytes:    62 6c 6f 62 20 36 00       (b l o b SP 6 NUL)

Full object:     62 6c 6f 62 20 36 00 68 65 6c 6c 6f 0a

SHA-1 of above → 8c7e5a667f1b771847fe88c01c3de34413a1b220  (40 chars)

Written to:      .git/objects/8c/7e5a667f1b771847fe88c01c3de34413a1b220
```

---

## 4. The Tree Object: A Binary Directory Snapshot

### 4.1 What a Tree Is (and Is Not)

A tree object is a snapshot of one directory at one point in time. It does not know about its parent directory. It does not know its own path. It only contains a list of its immediate children — each child described by three things:

- a **mode** (what kind of thing this is)
- a **name** (the filename or subdirectory name, no path)
- a **hash** (the SHA-1 address of the object this entry points to)

There are no parent pointers. The structure points strictly downward.

### 4.2 Mode Values

| Mode string | Meaning |
| ------------- | --------- |
| `100644` | Regular, non-executable file |
| `100755` | Executable file |
| `040000` | Directory (a nested tree object) |
| `120000` | Symbolic link |

For the `git-rs` implementation at this phase, only `100644` (regular files) and `040000` (directories) are used.

### 4.3 The Byte Format of a Single Entry

Each entry in a tree is serialized as:

```Text
[mode as ASCII bytes] [0x20] [name as bytes] [0x00] [20 raw hash bytes]
```

Broken down for a file named `sample.txt` with hash `1c2957419bb90febfe0c82b09dcbbed46fab37f1`:

```Text
31 30 30 36 34 34    → "100644" (6 bytes, ASCII)
20                   → space (1 byte, 0x20)
73 61 6d 70 6c 65 2e 74 78 74  → "sample.txt" (10 bytes, UTF-8)
00                   → null byte (1 byte, 0x00)
1c 29 57 41 9b b9 0f eb fe 0c  → raw SHA-1 bytes (20 bytes)
82 b0 9d cb be d4 6f ab 37 f1  → (continued)
```

Total for this one entry: 6 + 1 + 10 + 1 + 20 = **38 bytes**.

### 4.4 Why Raw Bytes, Not the Hex String

The 40-character hex string (`1c2957...`) is a human-readable representation of 20 bytes. Each byte becomes 2 hex characters. Git stores the raw 20 bytes instead of the hex string in tree entries.

Space difference for 100 entries:

| Representation | Bytes per hash | 100 entries |
| --------------- | --------------- | ------------- |
| 40-char hex string | 40 bytes | 4,000 bytes |
| 20 raw bytes | 20 bytes | 2,000 bytes |

Half the space, every time, for every hash pointer in every tree in the repository's entire history. In a repository with thousands of commits, this compounds significantly.

The conversion from hex string to raw bytes is a simple parse: take each pair of hex characters and convert to the byte value they represent. `"1c"` becomes `0x1c` (decimal 28). `"29"` becomes `0x29` (decimal 41). And so on for all 20 pairs.

### 4.5 Sorting: The Alphabetical Contract

Before the entries are serialized, they must be sorted by filename in **raw ASCII byte order**. This is not case-insensitive sorting. It is a strict, character-by-character comparison using each character's ASCII decimal value.

In ASCII, uppercase letters (65–90) come before lowercase letters (97–122). So `Readme.md` sorts before `src/`.

This sorting rule is part of Git's binary contract. Two implementations sorting the same set of files differently will produce different byte sequences, and therefore different SHA-1 hashes — even if the directory contents are identical. They will never interoperate.

Example with the actual test repo:

```Text
Files in folder1/:  new-sample.txt, file1.txt

ASCII sort order:
  'f' = 102
  'n' = 110

file1.txt sorts before new-sample.txt.
```

This is confirmed by the `git cat-file -p` output on the subdirectory's tree hash:

```Text
$ git cat-file -p 0ec3ee1930d31225f69076ad08a436b3a01d9908
100644 blob ...  file1.txt
100644 blob ...  new-sample.txt
```

### 4.6 The Full Tree Assembly Pipeline

Given a directory `folder1/` containing `file1.txt` and `new-sample.txt`:

```Text
Step 1: For each file, call write_object("blob", file_contents)
        → returns the blob hash for each file

Step 2: Build a list of TreeEntry {mode, name, hash} for each item

Step 3: Sort the list by name, raw ASCII order
        → ["file1.txt", "new-sample.txt"]

Step 4: Serialize to Vec<u8>:
        For each entry, append in order:
          mode.as_bytes()    → "100644"
          b' '               → space
          name.as_bytes()    → "file1.txt"
          0x00               → null byte
          hex_to_bytes(hash) → 20 raw bytes
        Repeat for each entry, all concatenated

Step 5: Call write_object("tree", &serialized_bytes)
        → internally: prepends "tree <size>\0", SHA-1s it, Zlib-compresses, writes to .git/objects
        → returns the tree hash

Step 6: Return the tree hash to the caller
```

---

## 5. Recursive Structure: How Trees Reference Trees

A subdirectory is represented as a tree entry with mode `040000` whose hash points to another tree object. The `write-tree` implementation handles this recursively: before the parent directory can be serialized, every subdirectory inside it must already have been processed and hashed.

This is not optional. A parent tree entry contains the hash of the child tree. The child tree's hash is derived from its content. You cannot know the child's hash until you have serialized the child. Therefore, the traversal must be **post-order**: leaves first, root last.

For the test repository:

```Tree
test-repo/
├── sample.txt
└── folder1/
    ├── file1.txt
    └── new-sample.txt
```

Processing order:

```Text
1. hash file1.txt → blob object written
2. hash new-sample.txt → blob object written
3. serialize folder1/ tree → tree object written
   hash: 0ec3ee1930d31225f69076ad08a436b3a01d9908
4. hash sample.txt → blob object written
5. serialize root tree → tree object written
   hash: 4d69c9857fa055de1b36ee9372f2b2bb92c844d5
```

Step 5 could not happen before step 3, because step 5 needs the hash from step 3 to write the `folder1` entry. The call stack, via recursion, enforces this ordering automatically.

---

## 6. The Interoperability Contract

The ultimate verification of a Git implementation is not that it compiles or runs without errors. It is that the official `git` binary — written in C, by different people, twenty years prior — can read the database your code wrote and agree on the content.

For Phase 3, the verification command is:

```bash
git cat-file -p <tree-hash>
```

When the hash produced by `git-rs write-tree` is passed to this command and the output matches the actual directory structure, the two implementations have agreed on the same bytes. There is no partial credit. Either the format is exact or the hash does not match and Git rejects it.

The test result for this implementation:

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

The root tree object correctly identifies `folder1` as a tree with its own hash, and `sample.txt` as a blob. The alphabetical ordering (`folder1` before `sample.txt`, since `'f'` = 102 < `'s'` = 115) is exact. Official Git read it without modification.

---

## 7. Transition to Commit Objects (Phase 4)

Tree objects represent directory snapshots, but they have no concept of *time*, *authorship*, or *history*. To create a versioned history, Git introduces **commit objects** — text-based objects that reference a tree (the snapshot), include author/committer metadata, and link to previous commits via parent references.

The transition from trees to commits is the transition from a *spatial* snapshot to a *temporal* history. See the Phase 4 documentation for the full treatment:

- [04_commit_object_and_commit_tree.md](04_commit_object_and_commit_tree.md) — Commit object format and DAG structure
- [05_dag_and_commit_serialization.md](05_dag_and_commit_serialization.md) — DAG mathematics and commit serialization
