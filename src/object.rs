use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

fn create_object(kind: &str, content: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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

fn compress_object(object: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(object)?;
    let compressed = compressor.finish();
    Ok(compressed?)
}

pub fn read_object(hash: &str) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    // 1. Resolve the filesystem path for this hash
    let path = object_path(hash);

    // 2. Read the compressed bytes from disk
    let compressed = fs::read(&path?)?;

    // 3. Decompress into a buffer
    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // 4. Locate the null byte separator
    let null_pos = decompressed
        .iter()
        .position(|&b| b == 0)
        .ok_or("Corrupt object: missing null separator")?;

    // 5. Split header and content using byte indices
    let header = std::str::from_utf8(&decompressed[..null_pos])?;
    let content = decompressed[null_pos + 1..].to_vec();

    // 6. Parse the header: "<kind> <size>"
    let mut parts = header.splitn(2, ' ');
    let kind = parts.next().ok_or("Corrupt object: missing object type")?;
    let declared_size: usize = parts
        .next()
        .ok_or("Corrupt object: missing size declaration")?
        .parse()?;

    // 7. Verify declared size matches actual content length (blueprint requirement)
    if declared_size != content.len() {
        return Err("Corrupt object: header size mismatch".into());
    }

    Ok((kind.to_string(), content))
}

fn object_path(hash: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let base = ".git/objects/";
    let file_name = hash.get(2..).ok_or("Invalid hash length")?;
    let dir = hash.get(..2).ok_or("Invalid hash length")?;
    let path = PathBuf::from(base).join(dir).join(file_name);
    Ok(path)
}

pub fn write_object(kind: &str, content: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let object = create_object(kind, content)?;
    let hashed_object = hash_object(&object);
    let compressed_object = compress_object(&object)?;
    let path = object_path(&hashed_object)?;
    fs::create_dir_all(&path.parent().ok_or("Invalid object path")?)?;
    fs::write(&path, compressed_object)?;
    Ok(hashed_object)
}
