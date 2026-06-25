# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-06-25

### Added

- **Porcelain `commit` command:** Added high-level `commit` command bridging the gap from plumbing to porcelain.
- **Refs Management:** Implemented branch pointer mutation and HEAD resolution.
- **Configurable Author:** Modified `commit` logic to read author and committer data from the `.git/config` file.

## [0.2.0] - 2026-06-24

### Added

- **Repo-Local Configuration:** Implemented `.git/config` parsing using `serde` and `toml` to allow isolated, project-specific user identities without relying on global system state.
- **Ecosystem Integration:** Replaced manual hex parsing with the highly optimized `hex` crate.

### Changed

- **Memory Efficiency:** Replaced `format!` + `extend_from_slice` patterns with the `write!` and `writeln!` macros across `commit.rs`, `tree.rs`, and `object.rs`. This writes formatted bytes directly into `Vec<u8>`, eliminating intermediate heap allocations.
- **Idiomatic Borrowing:** Updated the application boundary to accept borrowed slices (`&str`, `&[u8]`) instead of owned types (`String`, `&Vec<u8>`), drastically reducing `.clone()` calls and making the API more flexible.

### Fixed

- **Safety & Panic Prevention:** Replaced all `.unwrap()` calls and raw string slicing (`&hash[2..]`) with safe `.ok_or()?` and `.get()` methods. The CLI now gracefully handles edge cases like Detached HEAD states, malformed hashes, and missing parent directories instead of crashing.

### Style

- **Clippy Compliance:** Achieved 100% compliance with `cargo clippy -- -D warnings`, applying field init shorthand, removing redundant borrows, and standardizing buffer formatting.
