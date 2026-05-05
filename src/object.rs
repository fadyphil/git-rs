use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use std::io::{Read, Write};

pub fn create_blob(content: &[u8]) -> Vec<u8> {
    //here we create the vector that will hold the object which we will return
    let mut obj = Vec::new();
    //here is the way to append to the vector some ascii encoded bytes
    let size_str = content.len().to_string();

    obj.extend_from_slice(b"blob ");
    obj.extend_from_slice(size_str.as_bytes());
    obj.push(b'\0');
    obj.extend_from_slice(content);
    obj
}

pub fn hash_blob(blob: Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&blob);
    let hash = hasher.finalize();
    let hash_hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
    hash_hex
}

pub fn compress_blob(blob: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&blob)?;
    let compressed = compressor.finish();
    Ok(compressed?)
}

pub fn decompress_blob(compressed_blob: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut decompressor = ZlibDecoder::new(&compressed_blob[..]);
    let mut decompressed = Vec::new();
    decompressor.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}
