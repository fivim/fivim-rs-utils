use md5;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::{fs::File, io};

// Use sha2 crate to save memory, read file as stream, not read the entire file
pub fn sha256_by_file_path(file_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(file_path)?;

    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();
    let fh = format!("{:x}", hash);
    Ok(fh)
}

pub fn sha256_by_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let fh = format!("{:x}", hash);
    fh
}

pub fn crc32_by_bytes(bys: &[u8]) -> u32 {
    crc32fast::hash(bys)
}

pub fn crc32_of_sha256_by_data(data: &[u8]) -> u32 {
    crc32_by_bytes(sha256_by_bytes(data).as_bytes())
}

pub fn crc32_of_sha256_by_file_path(dist_path: &str) -> Result<u32, Box<dyn Error>> {
    let sha = sha256_by_file_path(dist_path)?;
    let crc = crc32_by_bytes(sha.as_bytes());
    Ok(crc)
}

pub fn md5_str(input: &str) -> [u8; 16] {
    *md5::compute(input)
}

pub fn md5_bytes(input: &Vec<u8>) -> [u8; 16] {
    *md5::compute(input)
}

#[test]
fn test_sha256_string() {
    let rrr = sha256_by_bytes("zxcvb".as_bytes());
    println!("sha256: {}", rrr);
}
#[test]
fn test_crc32_string() {
    let rrr = crc32_by_bytes("zxcvb".as_bytes());
    println!("crc32: {}", rrr);
}
