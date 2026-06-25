# Description

Please include a summary of the change and which issue is fixed. If this PR introduces a new architecture or modifies a critical storage mechanism, explain the *why* behind your approach.

Fixes # (issue)

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Refactor (memory/safety optimization)
- [ ] Documentation update
- [ ] Performance improvement

## Testing and Verification

Please describe the tests that you ran to verify your changes.
- [ ] I have verified this change operates correctly alongside the official `git` binary (e.g., `git cat-file -p <hash>`).
- [ ] I have added unit/integration tests that prove my fix is effective or my feature works.
- [ ] `cargo test` passes locally.

## Checklist

- [ ] My code follows the style guidelines of this project (`cargo fmt`).
- [ ] I have run `cargo clippy -- -D warnings` and fixed all warnings.
- [ ] I have added/updated strictly formatted Rustdoc comments (`///` or `//!`) detailing the *why* for any new/modified logic.
- [ ] I have updated the `CHANGELOG.md` with my changes (if applicable).
