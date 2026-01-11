use crate::error::AppError;
use crate::models::java::{JavaInstallation, JavaManifest};
use crate::utils::paths::{get_java_dir, get_java_manifest_path};
use crate::utils::platform::{Arch, Os};
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

const ADOPTIUM_API: &str = "https://api.adoptium.net/v3";

/// Determine the required Java major version for a Minecraft version
pub fn get_required_java_version(mc_version: &str) -> u32 {
    // Parse version like "1.21.4", "1.16.5", "24w10a" (snapshot)
    let parts: Vec<&str> = mc_version.split('.').collect();

    // Handle snapshots and other non-standard versions
    if parts.is_empty() || parts[0].chars().any(|c| c.is_alphabetic()) {
        // Snapshots and experimental versions - assume latest Java
        return 21;
    }

    let major = parts[0].parse::<u32>().unwrap_or(1);
    let minor = parts
        .get(1)
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|p| {
            // Handle versions like "1.20.5-pre1"
            p.split('-').next().and_then(|n| n.parse::<u32>().ok())
        })
        .unwrap_or(0);

    // MC 1.20.5+ requires Java 21
    if major > 1 || (major == 1 && minor > 20) || (major == 1 && minor == 20 && patch >= 5) {
        return 21;
    }

    // MC 1.17 - 1.20.4 requires Java 17
    if major == 1 && minor >= 17 {
        return 17;
    }

    // MC 1.16 and below uses Java 8
    8
}

/// Load the Java manifest from disk
pub fn load_java_manifest() -> JavaManifest {
    let manifest_path = get_java_manifest_path();

    if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str(&content) {
                return manifest;
            }
        }
    }

    JavaManifest::default()
}

/// Save the Java manifest to disk
pub fn save_java_manifest(manifest: &JavaManifest) -> Result<(), AppError> {
    let manifest_path = get_java_manifest_path();
    fs::create_dir_all(get_java_dir())?;

    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(&manifest_path, content)?;

    Ok(())
}

/// Check if a Java version is installed and return its path
pub fn get_installed_java(major_version: u32) -> Option<String> {
    let manifest = load_java_manifest();

    manifest
        .installations
        .iter()
        .find(|i| i.major_version == major_version)
        .and_then(|i| {
            // Verify the java executable still exists
            if PathBuf::from(&i.java_path).exists() {
                Some(i.java_path.clone())
            } else {
                None
            }
        })
}

/// Get the Adoptium API OS name
fn get_adoptium_os() -> &'static str {
    match Os::current() {
        Os::Windows => "windows",
        Os::MacOS => "mac",
        Os::Linux => "linux",
    }
}

/// Get the Adoptium API architecture name
fn get_adoptium_arch() -> &'static str {
    match Arch::current() {
        Arch::X64 => "x64",
        Arch::Arm64 => "aarch64",
        Arch::X86 => "x86",
    }
}

/// Build the Adoptium download URL for a Java version
fn get_adoptium_download_url(major_version: u32) -> String {
    let os = get_adoptium_os();
    let arch = get_adoptium_arch();

    format!(
        "{}/binary/latest/{}/ga/{}/{}/jdk/hotspot/normal/eclipse",
        ADOPTIUM_API, major_version, os, arch
    )
}

/// Ensure a Java version is installed, downloading if necessary
/// Returns the path to the java executable
pub async fn ensure_java_installed(
    major_version: u32,
    instance_id: &str,
    app_handle: &AppHandle,
) -> Result<String, AppError> {
    // Check if already installed
    if let Some(java_path) = get_installed_java(major_version) {
        return Ok(java_path);
    }

    // Need to download
    emit_status(
        app_handle,
        instance_id,
        &format!("Downloading Java {}...", major_version),
    );

    let url = get_adoptium_download_url(major_version);
    let client = reqwest::Client::new();

    // Send request
    let response =
        client.get(&url).send().await.map_err(|e| {
            AppError::JavaInstallError(format!("Failed to connect to Adoptium: {}", e))
        })?;

    if !response.status().is_success() {
        return Err(AppError::JavaInstallError(format!(
            "Adoptium returned status {}",
            response.status()
        )));
    }

    // Get content length for progress
    let total_size = response.content_length().unwrap_or(0);

    // Download to temp file
    let java_dir = get_java_dir();
    fs::create_dir_all(&java_dir)?;

    let extension = if Os::current() == Os::Windows {
        "zip"
    } else {
        "tar.gz"
    };
    let temp_path = java_dir.join(format!("temurin-{}-download.{}", major_version, extension));

    // Stream download with progress
    let mut file = fs::File::create(&temp_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| AppError::JavaInstallError(format!("Download error: {}", e)))?;
        file.write_all(&chunk)?;

        downloaded += chunk.len() as u64;

        // Emit progress
        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
            emit_status(
                app_handle,
                instance_id,
                &format!("Downloading Java {}... {}%", major_version, percent),
            );
        }
    }

    drop(file);

    // Extract
    emit_status(
        app_handle,
        instance_id,
        &format!("Extracting Java {}...", major_version),
    );

    let install_dir = java_dir.join(format!("temurin-{}", major_version));

    // Remove existing installation if present
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }

    fs::create_dir_all(&install_dir)?;

    // Extract based on OS
    if Os::current() == Os::Windows {
        extract_zip(&temp_path, &install_dir)?;
    } else {
        extract_tar_gz(&temp_path, &install_dir)?;
    }

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Find the java executable
    let java_path = find_java_executable(&install_dir)?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let java_exe = PathBuf::from(&java_path);
        if java_exe.exists() {
            let mut perms = fs::metadata(&java_exe)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&java_exe, perms)?;
        }
    }

    // Update manifest
    let mut manifest = load_java_manifest();

    // Remove any existing entry for this version
    manifest
        .installations
        .retain(|i| i.major_version != major_version);

    manifest.installations.push(JavaInstallation {
        major_version,
        java_path: java_path.clone(),
        installed_at: Utc::now().timestamp(),
    });

    save_java_manifest(&manifest)?;

    Ok(java_path)
}

/// Find the java executable within an extracted JDK directory
fn find_java_executable(install_dir: &PathBuf) -> Result<String, AppError> {
    let java_exe = if Os::current() == Os::Windows {
        "java.exe"
    } else {
        "java"
    };

    // Adoptium extracts to a directory like jdk-21.0.5+11
    // We need to find it and look in bin/

    for entry in fs::read_dir(install_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check for bin/java directly (Linux/Windows)
            let bin_java = path.join("bin").join(java_exe);
            if bin_java.exists() {
                return Ok(bin_java.to_string_lossy().to_string());
            }

            // macOS has Contents/Home/bin/java structure
            let mac_java = path
                .join("Contents")
                .join("Home")
                .join("bin")
                .join(java_exe);
            if mac_java.exists() {
                return Ok(mac_java.to_string_lossy().to_string());
            }
        }
    }

    Err(AppError::JavaInstallError(
        "Could not find java executable in extracted archive".to_string(),
    ))
}

/// Extract a .tar.gz archive
fn extract_tar_gz(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<(), AppError> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = fs::File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    archive
        .unpack(dest_dir)
        .map_err(|e| AppError::JavaInstallError(format!("Failed to extract tar.gz: {}", e)))?;

    Ok(())
}

/// Extract a .zip archive
fn extract_zip(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<(), AppError> {
    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

/// Emit a launch status update
fn emit_status(app_handle: &AppHandle, instance_id: &str, message: &str) {
    let _ = app_handle.emit(
        "launch_status",
        serde_json::json!({
            "instanceId": instance_id,
            "status": {
                "type": "preparing",
                "message": message
            }
        }),
    );
}
