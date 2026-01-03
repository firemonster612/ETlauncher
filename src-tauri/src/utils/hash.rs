use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Calculate SHA1 hash of a file
pub fn sha1_file(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify a file's SHA1 hash matches expected value
pub fn verify_sha1(path: &Path, expected: &str) -> std::io::Result<bool> {
    let actual = sha1_file(path)?;
    Ok(actual.eq_ignore_ascii_case(expected))
}

/// Calculate SHA1 hash of bytes
pub fn sha1_bytes(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
