# Build Git in Rust — Visual Reference

> Render with any Mermaid-compatible viewer: Obsidian, VS Code + Mermaid Preview,
> or paste individual blocks into mermaid.live.
> Each diagram uses the most semantically appropriate Mermaid dialect.

---

## 1 · Document Map

*The full knowledge structure of the blueprint — every part, section, and concept.*

```mindmap
mindmap
  root((Build Git in Rust))
    Part 0 Before You Begin
      Philosophy
        Not a VCS — a learning engine
        Smallest complete system
      Goals
        Binary that speaks real Git
        LLM Wiki from DAG
      Prerequisites
        rustup and cargo installed
        Read Rust Book chapters 1 to 3
    Part 1 Rust Prerequisites
      Memory Model
        Ownership — one owner per value
        Borrowing — shared and exclusive refs
        Slices — views into existing data
      Type System
        Structs and impl blocks
        Enums and exhaustive match
        Traits — Write Read Error
        Result T E and Option T
        Question mark operator
      Collections
        Vec of u8 — heap byte buffer
        String vs str
      Filesystem
        std fs — read write create
        PathBuf and Path
        CLI — std env args
    Part 2 What Git Actually Is
      Core Insight
        Git tracks content not filenames
        Content-addressable storage
      The dot git Directory
        objects — the database
        refs — named pointers
        HEAD — current branch pointer
      Cryptography
        SHA-1 — deterministic fixed-size ID
        Collision resistance
        One-way function
      Compression
        Zlib — lossless compression
        Hash before compress
      Four Object Types
        Blob — raw file bytes
        Tree — directory snapshot
        Commit — snapshot plus history
        Tag — named annotated ref
      The DAG
        Directed — child points to parent
        Acyclic — no loops in history
        Efficient — unchanged blobs reused
        Integrity — tampering changes all hashes
    Part 3 Object Format Bible
      Blob
        type SP size NUL content
        SHA-1 of full header plus content
        Zlib compressed on disk
      Tree
        Binary encoded entries
        mode SP name NUL 20 raw hash bytes
        Entries sorted alphabetically
      Commit
        Human readable text
        tree parent author committer message
        Unix timestamp and timezone
      Refs and HEAD
        HEAD points to branch name
        Branch file points to commit hash
        Update branch on each commit
    Part 4 Project Blueprint
      Cargo dot toml
        sha1 crate
        flate2 crate
        hex crate
      Directory Structure
        src main rs — CLI dispatcher
        src object rs — core read write hash
        src store rs — filesystem ops
        src tree rs — tree objects
        src commit rs — commit objects
        src refs rs — HEAD and refs
      Data Flow
        hash-object 7-step pipeline
        read minus compress minus strip minus return
    Part 5 Implementation
      Phase 1 init
        Create dot git skeleton
        Write HEAD file
        Verification — git status OK
      Phase 2 hash-object
        Read file bytes
        Format blob header
        SHA-1 hash
        Zlib compress
        Write to objects
        Verification — git cat-file reads blobs
      Phase 3 write-tree
        Walk directory recursively
        Skip dot git
        Sort entries
        Serialize binary entries
        Verification — git cat-file reads trees
      Phase 4 commit-tree
        Serialize commit text
        Store as commit object
        Verification — git cat-file reads commits
      Phase 5 refs and commit
        Read and write HEAD
        Update branch ref
        Full commit workflow
        Verification — git log shows history
      Phase 6 LLM Wiki
        export-snapshot command
        Walk commit to tree to blobs
        Emit structured JSON
        post-commit hook to local LLM
        Verification — wiki auto-generated
    Appendices
      Appendix A Retention Questions
        18 deep questions
        Answer before and after implementation
      Appendix B Verification Tests
        Per-phase table
        Cross-check with official git binary
```

---

## 2 · Phase Progression & Verification Gates

*The linear build sequence. Each phase unlocks the next. Gates are verified against the official `git` binary.*

```flowchart LR
flowchart LR
subgraph PH1["① init"]

ph1a["Create .git skeleton\nHEAD · objects · refs"]

end

  

subgraph PH2["② hash-object"]

ph2a["Blob storage\nSHA-1 + Zlib pipeline"]

end

  

subgraph PH3["③ write-tree"]

ph3a["Tree objects\nDirectory snapshot"]

end

  

subgraph PH4["④ commit-tree"]

ph4a["Commit objects\nTree + parent + message"]

end

  

subgraph PH5["⑤ commit"]

ph5a["Refs + HEAD\nFull commit workflow"]

end

  

subgraph PH6["⑥ LLM Wiki"]

ph6a["export-snapshot\nLocal LLM integration"]

end

  

PH1 --> PH2 --> PH3 --> PH4 --> PH5 --> PH6

  

PH1 -. "git status\nrecognizes repo" .-> G1(("✓"))

PH2 -. "git cat-file -p\nreads your blob" .-> G2(("✓"))

PH3 -. "git cat-file -p\nreads your tree" .-> G3(("✓"))

PH4 -. "git cat-file -p\nreads your commit" .-> G4(("✓"))

PH5 -. "git log --oneline\nshows your history" .-> G5(("✓"))

PH6 -. "docs/wiki/*.md\nauto-generated" .-> G6(("✓"))

  

style PH1 fill:#1a1a2e,stroke:#4a9eff,color:#fff

style PH2 fill:#1a1a2e,stroke:#4a9eff,color:#fff

style PH3 fill:#1a1a2e,stroke:#4a9eff,color:#fff

style PH4 fill:#1a1a2e,stroke:#4a9eff,color:#fff

style PH5 fill:#1a1a2e,stroke:#4a9eff,color:#fff

style PH6 fill:#1a1a2e,stroke:#4a9eff,color:#fff

style G1 fill:#2d6a2d,stroke:#4caf50,color:#fff

style G2 fill:#2d6a2d,stroke:#4caf50,color:#fff

style G3 fill:#2d6a2d,stroke:#4caf50,color:#fff

style G4 fill:#2d6a2d,stroke:#4caf50,color:#fff

style G5 fill:#2d6a2d,stroke:#4caf50,color:#fff

style G6 fill:#2d6a2d,stroke:#4caf50,color:#fff
```

---

## 3 · Module Architecture

*How the six source files depend on each other. Arrows mean "calls into".*

```flowchart LR
    flowchart LR
    CLI["**main.rs**\nCLI dispatcher\narg parsing · match dispatch"]

    OBJ["**object.rs**\nCore engine\nwrite_object · read_object\nhash · compress"]

    STORE["**store.rs**\nFilesystem ops\nobject_path · read_raw\nwrite_raw"]

    TREE["**tree.rs**\nTree objects\nwrite_tree · parse_tree\nserialize entries"]

    COM["**commit.rs**\nCommit objects\ncreate_commit · parse_commit\nCommitData struct"]

    REFS["**refs.rs**\nRefs and HEAD\nread_head · write_ref\ncurrent_commit · update_ref"]

    CLI --> OBJ
    CLI --> TREE
    CLI --> COM
    CLI --> REFS
    OBJ --> STORE
    TREE --> OBJ
    COM --> OBJ
    REFS --> STORE

    style CLI fill:#2c3e50,stroke:#e67e22,color:#fff
    style OBJ fill:#1a3a4a,stroke:#4a9eff,color:#fff
    style STORE fill:#1a3a4a,stroke:#4a9eff,color:#fff
    style TREE fill:#1a3a4a,stroke:#4a9eff,color:#fff
    style COM fill:#1a3a4a,stroke:#4a9eff,color:#fff
    style REFS fill:#1a3a4a,stroke:#4a9eff,color:#fff
```

---

## 4 · hash-object Pipeline

*The most important data flow in the project. Every other write operation is a variation of this seven-step sequence.*

```flowchart TD
flow chart TD
    A(["$ git-rs hash-object -w hello.txt"])

    A --> B["**1. READ**\nfs::read() → content: Vec of u8\n[104, 101, 108, 108, 111, 10]"]

    B --> C["**2. BUILD HEADER**\nformat! blob 6 NUL\nheader_bytes: Vec of u8"]

    B --> D_CONTENT["content bytes\nheld in memory"]

    C --> E["**3. CONCATENATE**\nfull_object = header_bytes + content\nVec::with_capacity pre-allocated"]
    D_CONTENT --> E

    E --> F["**4. HASH**\nSha1::new() → hasher.update(full_object)\nhasher.finalize() → 20 raw bytes\nhex-encode → 40-char string"]

    E --> G["**5. COMPRESS**\nZlibEncoder wraps Vec new\nencoder.write_all(full_object)\nencoder.finish() → compressed Vec of u8"]

    F --> H["**6. LOCATE PATH**\n.git/objects / XX / YYY...\nfirst 2 chars = dir\nremaining 38 = filename"]

    G --> I["compressed bytes\nready to write"]

    H --> J["**7. WRITE + PRINT**\nfs::create_dir_all(dir)\nfs::write(path, compressed)\nprintln! hash_hex"]
    I --> J

    J --> K(["hash: da39a3ee5e6b4b0d..."])

    style A fill:#2c3e50,stroke:#e67e22,color:#fff
    style K fill:#2d6a2d,stroke:#4caf50,color:#fff
    style F fill:#1a2e4a,stroke:#4a9eff,color:#fff
    style G fill:#1a2e4a,stroke:#4a9eff,color:#fff
    style E fill:#2a2a2a,stroke:#888,color:#fff
```

---

## 5 · Git Object Internal Formats

*The exact byte layout inside each object type — before SHA-1 and Zlib.*

```flowchart LR
flowchart LR
    subgraph SHELL["Every object on disk"]
        ZLIB(["Zlib compressed bytes"])
        ZLIB --> RAW["Decompressed:\ntype SPACE size NUL content"]
    end

    subgraph BLOB_FMT["Blob — stores a file"]
        BH["blob SPACE N NUL"]
        BC["raw file bytes\nexact content of file.txt"]
        BH --> BC
    end

    subgraph TREE_FMT["Tree — stores a directory"]
        TH["tree SPACE N NUL"]
        TE1["Entry 1:\nmode SPACE name NUL 20-raw-bytes"]
        TE2["Entry 2:\nmode SPACE name NUL 20-raw-bytes"]
        TEN["...sorted alphabetically..."]
        TH --> TE1 --> TE2 --> TEN
    end

    subgraph COMMIT_FMT["Commit — stores a snapshot"]
        CH["commit SPACE N NUL"]
        CL1["tree 40-char-hex-hash"]
        CL2["parent 40-char-hex-hash"]
        CL3["author name email timestamp tz"]
        CL4["committer name email timestamp tz"]
        CL5["blank line"]
        CL6["commit message text"]
        CH --> CL1 --> CL2 --> CL3 --> CL4 --> CL5 --> CL6
    end

    ZLIB -. "is one of" .-> BLOB_FMT
    ZLIB -. "is one of" .-> TREE_FMT
    ZLIB -. "is one of" .-> COMMIT_FMT

    style SHELL fill:#1a1a2e,stroke:#888,color:#fff
    style BLOB_FMT fill:#1a2e1a,stroke:#4caf50,color:#fff
    style TREE_FMT fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style COMMIT_FMT fill:#2e1a1a,stroke:#e67e22,color:#fff
```

---

## 6 · .git Directory Structure

*Every file Git creates and what it contains.*

```flowchart TD
flowchart TD
    ROOT[".git/"]

    ROOT --> OBJ[".git/objects/\nThe database — all stored objects"]
    ROOT --> REFS[".git/refs/\nNamed pointers to commit hashes"]
    ROOT --> HEAD_FILE[".git/HEAD\nref: refs/heads/main"]
    ROOT --> CONFIG[".git/config\nRepo configuration"]

    OBJ --> OBJ_INFO[".git/objects/info/\nEmpty in fresh repo"]
    OBJ --> OBJ_PACK[".git/objects/pack/\nEmpty in fresh repo"]
    OBJ --> OBJ_XX[".git/objects/XX/\n256 possible dirs, 00 to ff"]
    OBJ_XX --> OBJ_FILE[".git/objects/XX/YYY...\nCompressed object bytes\nFilename = remaining 38 hash chars"]

    REFS --> REFS_HEADS[".git/refs/heads/\nLocal branch pointers"]
    REFS --> REFS_TAGS[".git/refs/tags/\nTag pointers"]
    REFS_HEADS --> MAIN_FILE[".git/refs/heads/main\nda39a3ee...\n40-char commit hash"]

    style ROOT fill:#2c3e50,stroke:#e67e22,color:#fff
    style OBJ fill:#1a2e4a,stroke:#4a9eff,color:#fff
    style REFS fill:#1a2e1a,stroke:#4caf50,color:#fff
    style HEAD_FILE fill:#2e1a1a,stroke:#e74c3c,color:#fff
    style OBJ_FILE fill:#1a1a1a,stroke:#4a9eff,color:#ccc
    style MAIN_FILE fill:#1a1a1a,stroke:#4caf50,color:#ccc
```

---

## 7 · Git Object Relationships

*The entity model of Git's content-addressable database.*

```erDiagram
erDiagram
    COMMIT {
        string hash PK "40-char SHA-1 hex"
        string tree_hash FK "points to root Tree"
        string parent_hash FK "points to previous Commit"
        string author_name
        string author_email
        int unix_timestamp
        string timezone
        string message
    }

    TREE {
        string hash PK "40-char SHA-1 hex"
    }

    TREE_ENTRY {
        string mode "100644 100755 040000"
        string name "filename or dirname"
        bytes raw_hash_20 FK "20 raw bytes not 40 hex"
    }

    BLOB {
        string hash PK "40-char SHA-1 hex"
        bytes raw_content "exact file bytes"
    }

    REF {
        string name PK "e.g. refs/heads/main"
        string commit_hash FK
    }

    HEAD {
        string ref_path "e.g. refs/heads/main"
    }

    COMMIT ||--|| TREE : "points to (tree)"
    COMMIT ||--o| COMMIT : "parent (0 or 1)"
    TREE ||--|{ TREE_ENTRY : "contains (sorted)"
    TREE_ENTRY }o--o| BLOB : "may reference"
    TREE_ENTRY }o--o| TREE : "may reference (subdir)"
    REF ||--|| COMMIT : "points to"
    HEAD ||--|| REF : "indirects through"
```

---

## 8 · The Git DAG

*Directed Acyclic Graph — commits pointing backward through time. Unchanged blobs are shared across commits.*

```gitGraph
gitGraph
   commit id: "C1 Initial commit" tag: "tree-1"
   commit id: "C2 Add hello.txt" tag: "tree-2"
   commit id: "C3 Add world.txt" tag: "tree-3"
   branch feature-branch
   checkout feature-branch
   commit id: "C4 Feature work" tag: "tree-4"
   commit id: "C5 More feature" tag: "tree-5"
   checkout main
   commit id: "C6 Fix typo in hello.txt" tag: "tree-6"
   merge feature-branch id: "C7 Merge feature" tag: "tree-7"
   commit id: "C8 Release" tag: "v1.0"
```

---

## 9 · Git Object DAG — Blob Reuse

*How unchanged files are referenced across multiple commits without duplication.*

```flowchart LR
flowchart LR
    subgraph C1["Commit C1 — Initial"]
        T1["Tree 1"]
        B_readme["Blob: README.md\nhash: aaa..."]
        T1 --> B_readme
    end

    subgraph C2["Commit C2 — Add hello.txt"]
        T2["Tree 2"]
        B_readme2["Blob: README.md\nhash: aaa... SAME"]
        B_hello["Blob: hello.txt\nhash: bbb..."]
        T2 --> B_readme2
        T2 --> B_hello
    end

    subgraph C3["Commit C3 — Edit hello.txt"]
        T3["Tree 3"]
        B_readme3["Blob: README.md\nhash: aaa... SAME"]
        B_hello2["Blob: hello.txt\nhash: ccc... NEW"]
        T3 --> B_readme3
        T3 --> B_hello2
    end

    C1 --> C2 --> C3

    B_readme -.->|"one copy\non disk"| SHARED(("aaa...\nstored\nonce"))
    B_readme2 -.-> SHARED
    B_readme3 -.-> SHARED

    style SHARED fill:#2d6a2d,stroke:#4caf50,color:#fff
    style C1 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style C2 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style C3 fill:#1a1a2e,stroke:#4a9eff,color:#fff
```

---

## 10 · Rust Ownership Lifecycle

*The three states a value can be in. The compiler enforces all transitions at compile time — zero runtime cost.*

```stateDiagram-v2
stateDiagram-v2
    [*] --> Owned : let x = Value new

    Owned --> Moved : x passed to fn\nor assigned to y
    Moved --> [*] : new owner drops\n— data freed

    Owned --> SharedBorrow : borrow as ref x
    SharedBorrow --> SharedBorrow : another ref x\nmultiple OK
    SharedBorrow --> Owned : all borrows end\n— owner active again

    Owned --> ExclusiveBorrow : borrow as mut ref x
    ExclusiveBorrow --> Owned : borrow ends\n— owner active again

    Owned --> [*] : scope ends\n— drop called automatically

    note right of Owned
        One owner at all times
        Owner is responsible for drop
    end note

    note right of SharedBorrow
        Many readers allowed
        No writers while reading
    end note

    note right of ExclusiveBorrow
        Exactly one writer
        No other refs may exist
    end note

    note right of Moved
        Original variable invalid
        Compiler error if used
    end note
```

---

## 11 · Rust Key Concepts — Project Application Map

*How the Rust language features map directly to implementation tasks in this project.*

```flowchart TD
flowchart TD
    subgraph MEMORY["Memory Model"]
        OWN["Ownership\none value · one owner"]
        BORROW["Borrowing\n&T shared · &mut T exclusive"]
        SLICE["Slices\n&u8 view into Vec without copy"]
    end

    subgraph TYPES["Type System"]
        STRUCT["Structs\nGitObject · TreeEntry · CommitData"]
        ENUM["Enums\nCommand dispatch in main"]
        TRAIT["Traits\nWrite for ZlibEncoder\nRead for ZlibDecoder"]
        RESULT["Result + question mark\nerror propagation chain"]
    end

    subgraph STDLIB["Standard Library"]
        FS["std::fs\nread · write · create_dir_all"]
        PATH["PathBuf and Path\n.git/objects/XX/YYY"]
        CLI["std::env::args\nVec of String · match"]
    end

    subgraph CRATES["External Crates"]
        SHA["sha1 crate\nSha1::new · update · finalize"]
        ZLIB["flate2 crate\nZlibEncoder · ZlibDecoder"]
        HEX["hex crate\nraw bytes to hex string"]
    end

    OWN --> BORROW --> SLICE
    SLICE -->|"&u8 into hasher and encoder\nno extra allocation"| SHA
    SLICE -->|"&u8 into write_all"| ZLIB
    STRUCT -->|"GitObject holds kind + content"| FS
    ENUM -->|"match args to cmd functions"| CLI
    TRAIT -->|"impl Write for encoder"| ZLIB
    RESULT -->|"question mark on every file op"| FS
    PATH --> FS
    SHA -->|"20 bytes"| HEX
    HEX -->|"40-char hex"| PATH
    FS -->|".git/objects disk"| STORED(["Object stored on disk"])
    ZLIB -->|"compressed bytes"| FS

    style MEMORY fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style TYPES fill:#1a2e1a,stroke:#4caf50,color:#fff
    style STDLIB fill:#2e1a1a,stroke:#e67e22,color:#fff
    style CRATES fill:#2e2e1a,stroke:#f1c40f,color:#fff
    style STORED fill:#2d6a2d,stroke:#4caf50,color:#fff
```

---

## 12 · Full `commit` Command — Sequence Trace

*Every function call when you run `git-rs commit -m "First commit"`. The complete roundtrip.*

```sequenceDiagram
    sequenceDiagram
    actor User
    participant main
    participant tree
    participant object
    participant commit
    participant refs
    participant disk as "File System (.git)"

    User->>main: git-rs commit -m "First commit"

    main->>refs: current_commit()
    refs->>disk: read .git/HEAD
    disk-->>refs: "ref: refs/heads/main"
    refs->>disk: read .git/refs/heads/main
    disk-->>refs: None (first commit)
    refs-->>main: Option None

    main->>tree: write_tree(".")
    tree->>disk: read_dir(".")

    loop For each file (sorted, skipping .git)
        tree->>disk: fs::read(file)
        disk-->>tree: content: Vec of u8
        tree->>object: write_object("blob", content)
        object->>object: format header "blob N NUL"
        object->>object: concatenate full_object
        object->>object: SHA-1 hash full_object
        object->>object: Zlib compress full_object
        object->>disk: create_dir_all(".git/objects/XX")
        object->>disk: fs::write(".git/objects/XX/YYY", compressed)
        object-->>tree: blob_hash: String
    end

    tree->>object: write_object("tree", serialized_entries)
    Note over tree,object: entries = mode SP name NUL 20-raw-bytes per file
    object->>disk: write tree object to .git/objects
    object-->>tree: tree_hash: String
    tree-->>main: tree_hash: String

    main->>commit: create_commit(tree_hash, None, "First commit")
    commit->>commit: CommitData::new(tree, parent, msg)
    commit->>commit: serialize() → "tree X\nauthor...\n\nFirst commit"
    commit->>object: write_object("commit", content)
    object->>disk: write commit object to .git/objects
    object-->>commit: commit_hash: String
    commit-->>main: commit_hash: String

    main->>refs: update_current_ref(commit_hash)
    refs->>disk: read .git/HEAD → "ref: refs/heads/main"
    refs->>disk: write .git/refs/heads/main = commit_hash + newline
    refs-->>main: Ok

    main-->>User: [abc1234] First commit
```

---

## 13 · LLM Wiki Extension Architecture

*The translation layer between the binary Git object store and local LLM semantic processing.*

```flowchart TD
    flowchart TD
    subgraph WORK["Working Directory"]
        SRC["src/*.rs\nActual source files"]
    end

    subgraph GITRS["git-rs commit -m 'msg'"]
        STORE_OP["Compress + SHA-1\nall changed files"]
    end

    subgraph OBJDB[".git/objects — Binary Store"]
        BLOB_S["Zlib-compressed blobs\n(unreadable raw bytes)"]
        TREE_S["Zlib-compressed trees\n(binary entries)"]
        COM_S["Zlib-compressed commits\n(text headers)"]
    end

    subgraph EXPORT["git-rs export-snapshot"]
        E1["1. Read HEAD → branch"]
        E2["2. Read branch → commit hash"]
        E3["3. Read commit → tree hash"]
        E4["4. Walk tree recursively"]
        E5["5. Read each blob → decompress → strip header"]
        E6["6. Emit structured JSON"]
        E1 --> E2 --> E3 --> E4 --> E5 --> E6
    end

    subgraph JSON_OUT["snapshot.json"]
        J1["commit: hash"]
        J2["message: text"]
        J3["files: path + content"]
    end

    subgraph LLM_PROC["Local LLM — Ollama / Mistral / GLM"]
        L1["Architecture summary"]
        L2["Key functions and data flow"]
        L3["Changes since last commit"]
        L4["Open questions and TODOs"]
    end

    subgraph WIKI["docs/wiki/DATE.md"]
        W1["Living codebase knowledge base"]
    end

    SRC --> GITRS --> OBJDB
    OBJDB --> EXPORT
    EXPORT --> JSON_OUT
    JSON_OUT --> LLM_PROC
    LLM_PROC --> WIKI

    HOOK["post-commit hook\nauto-runs after every commit"]
    GITRS --> HOOK
    HOOK --> EXPORT

    style WORK fill:#1a2e1a,stroke:#4caf50,color:#fff
    style GITRS fill:#2c3e50,stroke:#e67e22,color:#fff
    style OBJDB fill:#1a1a1a,stroke:#888,color:#ccc
    style EXPORT fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style JSON_OUT fill:#2e2e1a,stroke:#f1c40f,color:#fff
    style LLM_PROC fill:#2e1a2e,stroke:#9b59b6,color:#fff
    style WIKI fill:#2d6a2d,stroke:#4caf50,color:#fff
    style HOOK fill:#2c3e50,stroke:#e67e22,color:#fff
```

---

## 14 · Verification Test Matrix

*Cross-reference: your binary output vs official `git` binary. Each row is a quality gate.*

```flowchart LR
    subgraph V1["Phase 1 — init"]
        v1a["git-rs init"] --> v1b["git status\nrecognizes empty repo"]
        v1a --> v1c["cat .git/HEAD\nref: refs/heads/main"]
    end

    subgraph V2["Phase 2 — hash-object"]
        v2a["echo hello > f.txt\ngit-rs hash-object -w f.txt\n→ prints 40-char hash"] --> v2b["git cat-file -t HASH\n→ blob"]
        v2b --> v2c["git cat-file -p HASH\n→ hello"]
        v2c --> v2d["git-rs cat-file -p HASH\n→ hello"]
    end

    subgraph V3["Phase 3 — write-tree"]
        v3a["git-rs write-tree\n→ prints tree hash"] --> v3b["git cat-file -p TREE\n→ file listing"]
        v3b --> v3c["git-rs ls-tree TREE\n→ same listing"]
    end

    subgraph V4["Phase 4 — commit-tree"]
        v4a["git-rs commit-tree TREE -m msg\n→ commit hash"] --> v4b["git cat-file -p COMMIT\n→ tree + author + message"]
    end

    subgraph V5["Phase 5 — commit"]
        v5a["git-rs commit -m first\n→ abc1234 first"] --> v5b["git log --oneline\n→ commit in history"]
        v5b --> v5c["git-rs commit -m second\ngit log shows both"]
    end

    subgraph V6["Phase 6 — export-snapshot"]
        v6a["git-rs export-snapshot\n→ valid JSON"] --> v6b["pipe to LLM\n→ wiki page generated"]
    end

    V1 --> V2 --> V3 --> V4 --> V5 --> V6

    style V1 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style V2 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style V3 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style V4 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style V5 fill:#1a1a2e,stroke:#4a9eff,color:#fff
    style V6 fill:#1a1a2e,stroke:#4a9eff,color:#fff
```

---

## 15 · Retention Questions — Concept Map

*The 18 deep questions from Appendix A organized by domain — for spaced repetition review.*

```mindmap
root((Retention Questions))
  Rust Language
    Ownership and borrowing
      String vs ref String vs str vs Vec u8 vs ref u8
      When does Rust free a value
      Why cant you have mut ref and ref simultaneously
    Error handling
      Expand let x = thing question-mark into full match
      Why does Box dyn Error exist
      Explain collect Result Vec u8 step by step
    Iterators
      iter vs into_iter vs iter_mut — ownership difference
    Performance
      Vec with_capacity vs Vec new — why it matters
  Git Internals
    Storage model
      What is content-addressable storage
      Two SHA-1 properties that make it suitable
    Blob format
      Exact bytes for content hi newline
    Tree format
      Why 20-byte raw hashes not 40-char hex
      Space difference for 100 entries
    DAG
      What is the DAG and why is it acyclic
      What would a cycle mean
    History integrity
      How does Git detect retroactive tampering
    Pipeline
      Where in the pipeline is Zlib applied
      Why after hashing not before
  Architecture
    Data flow
      Draw the full commit -m flow
      Every function and every file touched
    LLM integration
      Why is export-snapshot architecturally necessary
      Why cant you point LLM at dot git directly
    Extensions
      How would you implement git diff
      Which two objects would you compare
```

---

*Document version 1.0 — May 2026*
*14 diagrams · 6 Mermaid types: mindmap · flowchart · gitGraph · erDiagram · stateDiagram-v2 · sequenceDiagram*
*Render online: paste any block into mermaid.live — convert to Excalidraw via mermaid-to-excalidraw*
