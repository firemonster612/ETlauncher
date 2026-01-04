use rayon::prelude::*;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest as Sha2Digest, Sha512};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

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
        Sha1Digest::update(&mut hasher, &buffer[..bytes_read]);
    }

    Ok(format!("{:x}", Sha1Digest::finalize(hasher)))
}

/// Calculate SHA512 hash of a file
pub fn sha512_file(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha512::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        Sha2Digest::update(&mut hasher, &buffer[..bytes_read]);
    }

    Ok(format!("{:x}", Sha2Digest::finalize(hasher)))
}

/// Verify a file's SHA1 hash matches expected value
pub fn verify_sha1(path: &Path, expected: &str) -> std::io::Result<bool> {
    let actual = sha1_file(path)?;
    Ok(actual.eq_ignore_ascii_case(expected))
}

/// Calculate SHA1 hash of bytes
pub fn sha1_bytes(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    Sha1Digest::update(&mut hasher, data);
    format!("{:x}", Sha1Digest::finalize(hasher))
}

/// Calculate Murmur2 fingerprint for CurseForge compatibility
/// Strips whitespace (tab, lf, cr, space) before hashing as required by CurseForge
pub fn murmur2_bytes(data: &[u8]) -> u32 {
    // CurseForge fingerprint uses murmur2 on file content with whitespace stripped
    // Strip: tab (9), lf (10), cr (13), space (32)
    let filtered: Vec<u8> = data
        .iter()
        .copied()
        .filter(|&b| b != 9 && b != 10 && b != 13 && b != 32)
        .collect();
    murmur2::murmur2(&filtered, 1)
}

/// Result of hashing a file with multiple algorithms
#[derive(Debug, Clone)]
pub struct FileHashResult {
    pub path: PathBuf,
    pub sha512: String,
    pub murmur2_fingerprint: u32,
    pub size: u64,
}

/// Calculate SHA512 and Murmur2 hashes for a single file
/// The Murmur2 fingerprint is computed on file content with whitespace stripped
/// (as required by CurseForge fingerprinting)
fn hash_file_with_fingerprint(path: &Path) -> std::io::Result<FileHashResult> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();

    let mut reader = BufReader::new(file);
    let mut sha512_hasher = Sha512::new();
    let mut all_bytes = Vec::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        Sha2Digest::update(&mut sha512_hasher, &buffer[..bytes_read]);
        all_bytes.extend_from_slice(&buffer[..bytes_read]);
    }

    let sha512 = format!("{:x}", Sha2Digest::finalize(sha512_hasher));

    // CurseForge fingerprint uses murmur2 on file content with whitespace stripped
    // Strip: tab (9), lf (10), cr (13), space (32)
    let filtered: Vec<u8> = all_bytes
        .into_iter()
        .filter(|&b| b != 9 && b != 10 && b != 13 && b != 32)
        .collect();
    let murmur2_fingerprint = murmur2::murmur2(&filtered, 1);

    Ok(FileHashResult {
        path: path.to_path_buf(),
        sha512,
        murmur2_fingerprint,
        size,
    })
}

/// Calculate SHA512 and Murmur2 hashes for multiple files in parallel using rayon
pub fn hash_files_parallel(paths: &[PathBuf]) -> Vec<Result<FileHashResult, std::io::Error>> {
    paths
        .par_iter()
        .map(|path| hash_file_with_fingerprint(path))
        .collect()
}
