use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObjectError {
    // --- STANDARD LIBRARY ERRORS (Using #[from]) ---
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 parsing error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Integer parsing error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    // --- CUSTOM DOMAIN LOGIC ERRORS ---
    #[error("Corrupt object: missing null separator")]
    MissingNullSeparator,

    #[error("Corrupt object: missing object type")]
    MissingObjectType,

    #[error("Corrupt object: missing size declaration")]
    MissingSizeDeclaration,

    // Notice how we can hold data inside the variant!
    #[error("Corrupt object: header size mismatch (expected {expected}, got {actual})")]
    SizeMismatch { expected: usize, actual: usize },

    #[error("Invalid hash length: must be at least 40 characters")]
    InvalidHashLength,

    #[error("Invalid object path: no parent directory found")]
    InvalidObjectPath,
}

fn create_object(kind: &str, content: &[u8]) -> Result<Vec<u8>, ObjectError> {
    //here we create the vector that will hold the object which we will return
    let mut obj = Vec::new();
    //here is the way to append to the vector some ascii encoded bytes
    write!(&mut obj, "{} {}\0", kind, content.len())?;
    obj.extend_from_slice(content);
    Ok(obj)
}

fn hash_object(object: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(object);
    let hash = hasher.finalize();
    let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    hash_hex
}

fn compress_object(object: &[u8]) -> Result<Vec<u8>, ObjectError> {
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(object)?;
    let compressed = compressor.finish();
    Ok(compressed?)
}

pub fn read_object(hash: &str, dir: &Path) -> Result<(String, Vec<u8>), ObjectError> {
    // 1. Resolve the filesystem path for this hash
    let path = object_path(hash, dir)?;

    // 2. Read the compressed bytes from disk
    let compressed = fs::read(&path)?;

    // 3. Decompress into a buffer
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // 4. Locate the null byte separator
    let null_pos = decompressed
        .iter()
        .position(|&b| b == 0)
        .ok_or(ObjectError::MissingNullSeparator)?;

    // 5. Split header and content using byte indices
    let header = std::str::from_utf8(&decompressed[..null_pos])?;
    let content = decompressed[null_pos + 1..].to_vec();

    // 6. Parse the header: "<kind> <size>"
    let mut parts = header.splitn(2, ' ');
    let kind = parts.next().ok_or(ObjectError::MissingObjectType)?;
    let declared_size: usize = parts
        .next()
        .ok_or(ObjectError::MissingSizeDeclaration)?
        .parse()?;

    // 7. Verify declared size matches actual content length (blueprint requirement)
    if declared_size != content.len() {
        return Err(ObjectError::SizeMismatch {
            expected: declared_size,
            actual: content.len(),
        });
    }

    Ok((kind.to_string(), content))
}

fn object_path(hash: &str, repo_dir: &Path) -> Result<PathBuf, ObjectError> {
    if hash.len() != 40 {
        return Err(ObjectError::InvalidHashLength);
    }
    let base = repo_dir.join(".git").join("objects");
    let file_name = hash.get(2..).ok_or(ObjectError::InvalidHashLength)?;
    let dir = hash.get(..2).ok_or(ObjectError::InvalidHashLength)?;
    let path = PathBuf::from(base).join(dir).join(file_name);
    Ok(path)
}

pub fn write_object(kind: &str, content: &[u8], dir: &Path) -> Result<String, ObjectError> {
    let object = create_object(kind, content)?;
    let hashed_object = hash_object(&object);
    let compressed_object = compress_object(&object)?;
    let path = object_path(&hashed_object, dir)?;
    fs::create_dir_all(path.parent().ok_or(ObjectError::InvalidObjectPath)?)?;
    fs::write(path, compressed_object)?;
    Ok(hashed_object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // --- PURE TESTS ---
    #[test]
    fn test_create_object_formatting() {
        let result = create_object("blob", b"test content").unwrap();
        let mut expected = b"blob 12\0".to_vec();
        expected.extend_from_slice(b"test content");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_object_empty() {
        let result = create_object("blob", b"").unwrap();
        assert_eq!(result, b"blob 0\0".to_vec());
    }

    #[test]
    fn test_hash_object_known_value() {
        let obj = b"blob 0\0".to_vec();
        let hash = hash_object(&obj);
        assert_eq!(hash, "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }

    #[test]
    fn test_object_path_valid() {
        let dir = tempdir().unwrap();
        let hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let path = object_path(hash, dir.path()).unwrap();
        assert_eq!(
            path,
            dir.path()
                .join(".git/objects/a1/b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        );
    }

    #[test]
    fn test_object_path_too_short() {
        let dir = tempdir().unwrap();
        let result = object_path("abc", dir.path());
        assert!(matches!(
            result.unwrap_err(),
            ObjectError::InvalidHashLength
        ));
    }

    // --- IMPURE TESTS (Now completely isolated!) ---
    #[test]
    fn test_write_and_read_roundtrip() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".git/objects")).unwrap();

        let content = b"hello world";
        let hash = write_object("blob", content, dir.path()).unwrap();
        let (kind, read_content) = read_object(&hash, dir.path()).unwrap();

        assert_eq!(kind, "blob");
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_object_corrupt_missing_null() {
        let dir = tempdir().unwrap();
        let hash = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let path = object_path(hash, dir.path()).unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
        compressor.write_all(b"blob 5hello").unwrap();
        fs::write(path, compressor.finish().unwrap()).unwrap();

        let result = read_object(hash, dir.path());
        assert!(matches!(
            result.unwrap_err(),
            ObjectError::MissingNullSeparator
        ));
    }

    #[test]
    fn test_read_object_corrupt_size_mismatch() {
        let dir = tempdir().unwrap();
        let hash = "b1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
        let path = object_path(hash, dir.path()).unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
        compressor.write_all(b"blob 99\0hello").unwrap();
        fs::write(path, compressor.finish().unwrap()).unwrap();

        let result = read_object(hash, dir.path());
        assert!(matches!(
            result.unwrap_err(),
            ObjectError::SizeMismatch { .. }
        ));
    }
}
