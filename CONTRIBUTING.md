# Contributing to git-rs

First off, thank you for considering contributing to `git-rs`! It's a learning vehicle for systems programming, and contributions that help demystify Git's internals are highly welcome.

## How Can I Contribute?

### Reporting Bugs

- Ensure the bug was not already reported by searching [Issues](../../issues).
- If you're unable to find an open issue addressing the problem, [open a new one](../../issues/new). Be sure to include a clear title, a detailed description, and exact steps to reproduce the issue.

### Suggesting Enhancements

- Open an issue describing the Git feature you'd like to see implemented (e.g., `merge`, `rebase`, or new plumbing commands).

### Pull Requests

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. **Crucial:** Ensure your code passes our strict linting and formatting checks.

## Development Workflow & Code Standards

This project maintains a strict standard for idiomatic Rust. Before submitting a PR, you **must** run the following commands locally:

```bash
# 1. Format all code to standard Rust style
cargo fmt

# 2. Run the linter. We treat warnings as errors.
cargo clippy -- -D warnings

# 3. Ensure all tests pass
cargo test
```

> If `cargo clippy` throws a warning, your PR will not be merged until it is resolved. We prioritize memory efficiency (zero intermediate allocations), safe error propagation (no `.unwrap()`), and idiomatic borrowing (`&str` over `String`).

## Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/?spm=a2ty_o01.29997173.0.0.689d55fbfEbklJ). Your commit messages must follow this format:

```bash
<type>(<scope>): <subject>

<body>
```

### Types

- `feat`: A new feature

- `fix`: A bug fix

- `refactor`: A code change that neither fixes a bug nor adds a feature (e.g., memory optimizations)

- `docs`: Documentation only changes

- `style`: Changes that do not affect the meaning of the code (white-space, formatting)

- `test`:  Adding missing tests or correcting existing tests

### Example

```bash
refactor(object): eliminate intermediate heap allocations in create_object

Replaced format!().as_bytes() + extend_from_slice patterns with the write! macro.
```

## Continuous Integration (CI)

All Pull Requests are automatically tested against our GitHub Actions pipeline. To ensure your PR is green, run the following locally before pushing:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
