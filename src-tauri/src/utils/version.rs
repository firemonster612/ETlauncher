//! Version comparison utilities for Minecraft and mod versions
//!
//! Provides functions to parse, compare, and validate version strings.

use std::cmp::Ordering;

/// Parsed Minecraft version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    /// Pre-release suffix (e.g., "pre1", "rc1")
    pub prerelease: Option<String>,
    /// Snapshot identifier (e.g., "24w14a")
    pub snapshot: Option<String>,
}

/// Parse a Minecraft version string into components
///
/// Handles formats like:
/// - "1.21.4" -> (1, 21, 4)
/// - "1.21" -> (1, 21, 0)
/// - "1.20.1-pre1" -> (1, 20, 1) with prerelease
/// - "24w14a" -> snapshot
pub fn parse_mc_version(version: &str) -> Option<McVersion> {
    let version = version.trim();

    // Check if it's a snapshot (e.g., "24w14a")
    if version.len() >= 5 && version.chars().take(2).all(|c| c.is_ascii_digit())
        && version.chars().nth(2) == Some('w')
    {
        return Some(McVersion {
            major: 1,
            minor: 0,
            patch: 0,
            prerelease: None,
            snapshot: Some(version.to_string()),
        });
    }

    // Split off pre-release suffix
    let (version_part, prerelease) = if let Some(idx) = version.find('-') {
        (&version[..idx], Some(version[idx + 1..].to_string()))
    } else {
        (version, None)
    };

    let parts: Vec<&str> = version_part.split('.').collect();

    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let major = parts.first()?.parse().ok()?;
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    Some(McVersion {
        major,
        minor,
        patch,
        prerelease,
        snapshot: None,
    })
}

/// Compare two Minecraft versions
///
/// Returns Ordering based on which version is newer.
/// Snapshots are considered older than releases.
/// Pre-releases are considered older than full releases of the same version.
pub fn compare_mc_versions(a: &str, b: &str) -> Ordering {
    let a_parsed = parse_mc_version(a);
    let b_parsed = parse_mc_version(b);

    match (a_parsed, b_parsed) {
        (Some(a), Some(b)) => {
            // Compare snapshots specially
            match (&a.snapshot, &b.snapshot) {
                (Some(a_snap), Some(b_snap)) => return a_snap.cmp(b_snap),
                (Some(_), None) => return Ordering::Less, // Snapshot < Release
                (None, Some(_)) => return Ordering::Greater,
                (None, None) => {}
            }

            // Compare version numbers
            match a.major.cmp(&b.major) {
                Ordering::Equal => {}
                other => return other,
            }
            match a.minor.cmp(&b.minor) {
                Ordering::Equal => {}
                other => return other,
            }
            match a.patch.cmp(&b.patch) {
                Ordering::Equal => {}
                other => return other,
            }

            // Compare pre-release (None > Some)
            match (&a.prerelease, &b.prerelease) {
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a_pre), Some(b_pre)) => a_pre.cmp(b_pre),
                (None, None) => Ordering::Equal,
            }
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => a.cmp(b), // Fallback to string comparison
    }
}

/// Check if version B is newer than version A
///
/// Handles various version formats:
/// - Semantic versions (1.2.3, 1.2.3-beta)
/// - Mod versions (0.15.11, 4.0.0+build.1)
/// - Timestamp-based versions (20240115)
pub fn is_version_newer(current: &str, candidate: &str) -> bool {
    // First try semantic version comparison
    if let (Some(current_nums), Some(candidate_nums)) = (
        parse_version_numbers(current),
        parse_version_numbers(candidate),
    ) {
        for (c, n) in current_nums.iter().zip(candidate_nums.iter()) {
            match n.cmp(c) {
                Ordering::Greater => return true,
                Ordering::Less => return false,
                Ordering::Equal => continue,
            }
        }
        // If all compared parts are equal, longer version with more parts might be newer
        if candidate_nums.len() > current_nums.len() {
            // Check if extra parts are non-zero
            for &n in candidate_nums.iter().skip(current_nums.len()) {
                if n > 0 {
                    return true;
                }
            }
        }
        return false;
    }

    // Fallback: lexicographic comparison (for edge cases)
    candidate > current
}

/// Parse version string into numeric components
///
/// Handles formats like:
/// - "1.2.3" -> [1, 2, 3]
/// - "1.2.3-beta.1" -> [1, 2, 3, 1]
/// - "1.2.3+build.5" -> [1, 2, 3] (build metadata ignored)
fn parse_version_numbers(version: &str) -> Option<Vec<u64>> {
    let version = version.trim();

    // Remove build metadata (everything after +)
    let version = version.split('+').next().unwrap_or(version);

    // Split by common delimiters and extract numbers
    let mut numbers = Vec::new();

    for part in version.split(|c: char| c == '.' || c == '-' || c == '_') {
        // Try to parse as number, skip non-numeric parts (like "beta", "alpha")
        if let Ok(num) = part.parse::<u64>() {
            numbers.push(num);
        } else if part.chars().take_while(|c| c.is_ascii_digit()).count() > 0 {
            // Extract leading digits from parts like "1a", "2beta"
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = digits.parse::<u64>() {
                numbers.push(num);
            }
        }
    }

    if numbers.is_empty() {
        None
    } else {
        Some(numbers)
    }
}

/// Check if a version string looks like a valid Minecraft version
pub fn is_valid_mc_version(version: &str) -> bool {
    parse_mc_version(version).is_some()
}

/// Get major.minor version for loader compatibility checking
///
/// Returns "1.21" from "1.21.4"
pub fn get_mc_major_minor(version: &str) -> Option<String> {
    let parsed = parse_mc_version(version)?;
    if parsed.snapshot.is_some() {
        return None;
    }
    Some(format!("{}.{}", parsed.major, parsed.minor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mc_version() {
        let v = parse_mc_version("1.21.4").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 21);
        assert_eq!(v.patch, 4);

        let v = parse_mc_version("1.20").unwrap();
        assert_eq!(v.patch, 0);

        let v = parse_mc_version("1.20.1-pre1").unwrap();
        assert_eq!(v.prerelease, Some("pre1".to_string()));
    }

    #[test]
    fn test_compare_mc_versions() {
        assert_eq!(compare_mc_versions("1.21.4", "1.21.3"), Ordering::Greater);
        assert_eq!(compare_mc_versions("1.20.1", "1.21.0"), Ordering::Less);
        assert_eq!(compare_mc_versions("1.20.1-pre1", "1.20.1"), Ordering::Less);
    }

    #[test]
    fn test_is_version_newer() {
        assert!(is_version_newer("1.0.0", "1.0.1"));
        assert!(is_version_newer("1.0.0", "2.0.0"));
        assert!(!is_version_newer("1.0.1", "1.0.0"));
        assert!(!is_version_newer("1.0.0", "1.0.0"));

        // Mod versions
        assert!(is_version_newer("0.15.10", "0.15.11"));
        assert!(is_version_newer("4.0.0", "4.0.1-beta"));
    }
}
