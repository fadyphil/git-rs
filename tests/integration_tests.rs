use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_full_git_workflow() {
    // 1. ARRANGE: Create an isolated, ephemeral filesystem
    // `tempdir()` creates a unique folder in the OS's temp directory.
    // It is automatically deleted when the `dir` variable goes out of scope.
    let dir = tempdir().expect("Failed to create temp dir");
    let dir_path = dir.path();

    // Create a dummy file to track
    let file_path = dir_path.join("hello.txt");
    fs::write(&file_path, "Hello, git-rs!").expect("Failed to write dummy file");

    // 2. ACT & ASSERT: Initialize the repository
    // We use `.current_dir(dir_path)` to force the binary to run INSIDE our temp folder,
    // completely isolating it from your real project files.
    Command::cargo_bin("git-rs")
        .unwrap()
        .current_dir(dir_path)
        .arg("init")
        .assert()
        .success();

    // Overwrite the config to ensure deterministic author identity for the test
    let config_path = dir_path.join(".git").join("config");
    fs::write(
        &config_path,
        "[user]\nname = \"Test User\"\nemail = \"test@example.com\"\n",
    )
    .expect("Failed to write mock git config");

    // 3. ACT & ASSERT: Hash the object
    let hash_output = Command::cargo_bin("git-rs")
        .unwrap()
        .current_dir(dir_path)
        .args(["hash-object", "-w", "hello.txt"])
        .output()
        .expect("Failed to execute hash-object");

    assert!(hash_output.status.success(), "hash-object failed");
    let hash = String::from_utf8_lossy(&hash_output.stdout);
    let hash = hash.trim();

    // Verify the hash is exactly 40 hex characters
    assert_eq!(hash.len(), 40, "Object hash should be 40 chars");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Invalid hex character"
    );

    // 4. ACT & ASSERT: Write the tree
    let tree_output = Command::cargo_bin("git-rs")
        .unwrap()
        .current_dir(dir_path)
        .arg("write-tree")
        .output()
        .expect("Failed to execute write-tree");

    assert!(tree_output.status.success(), "write-tree failed");
    let tree_hash = String::from_utf8_lossy(&tree_output.stdout)
        .trim()
        .to_string();
    assert_eq!(tree_hash.len(), 40);

    // 5. ACT & ASSERT: Create the commit
    let commit_output = Command::cargo_bin("git-rs")
        .unwrap()
        .current_dir(dir_path)
        .args(["commit", "-m", "Initial test commit"])
        .output()
        .expect("Failed to execute commit");

    assert!(
        commit_output.status.success(),
        "commit failed: {}",
        String::from_utf8_lossy(&commit_output.stderr)
    );
    let commit_hash = String::from_utf8_lossy(&commit_output.stdout)
        .trim()
        .to_string();
    assert_eq!(commit_hash.len(), 40);

    // 6. FINAL VERIFICATION: Check the Ref Update
    // Prove that `update_current_ref` actually wrote the hash to the correct file
    let ref_path = dir_path
        .join(".git")
        .join("refs")
        .join("heads")
        .join("main");
    let stored_ref = fs::read_to_string(ref_path).expect("Failed to read HEAD ref");
    assert_eq!(
        stored_ref.trim(),
        commit_hash,
        "The branch reference was not updated to the new commit hash"
    );
}
