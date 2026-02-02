/// Operating system types for Minecraft rules
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
}

impl Os {
    /// Get the current operating system
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Os::Windows;
        #[cfg(target_os = "macos")]
        return Os::MacOS;
        #[cfg(target_os = "linux")]
        return Os::Linux;
    }

    /// Get the OS name as used in Minecraft version JSON
    pub fn as_str(&self) -> &'static str {
        match self {
            Os::Windows => "windows",
            Os::MacOS => "osx",
            Os::Linux => "linux",
        }
    }
}

impl std::fmt::Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// CPU architecture types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arch {
    X86,
    X64,
    Arm64,
}

impl Arch {
    /// Get the current architecture
    pub fn current() -> Self {
        #[cfg(target_arch = "x86")]
        return Arch::X86;
        #[cfg(target_arch = "x86_64")]
        return Arch::X64;
        #[cfg(target_arch = "aarch64")]
        return Arch::Arm64;
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        return Arch::X64; // Default fallback
    }

    /// Get the architecture string as used in Minecraft natives
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X86 => "32",
            Arch::X64 => "64",
            Arch::Arm64 => "arm64",
        }
    }
}

/// Get the classpath separator for the current OS
pub fn classpath_separator() -> &'static str {
    #[cfg(target_os = "windows")]
    return ";";
    #[cfg(not(target_os = "windows"))]
    return ":";
}

/// Get the native library extension for the current OS
pub fn native_lib_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    return ".dll";
    #[cfg(target_os = "macos")]
    return ".dylib";
    #[cfg(target_os = "linux")]
    return ".so";
}

/// Get the current OS name as used in Minecraft rules
pub fn get_os_name() -> &'static str {
    Os::current().as_str()
}

/// Get the current architecture as used in Minecraft rules
pub fn get_arch() -> &'static str {
    match Arch::current() {
        Arch::X86 => "x86",
        Arch::X64 => "x86_64",
        Arch::Arm64 => "aarch64",
    }
}

/// Convert a path to its short (8.3) form on Windows to avoid command line length limits.
/// On non-Windows platforms, returns the path string unchanged.
#[cfg(target_os = "windows")]
pub fn to_short_path(path: &std::path::Path) -> String {
    use std::os::windows::ffi::OsStrExt;

    // Convert path to wide string (UTF-16) for Windows API
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // First call to get required buffer size
    let len = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetShortPathNameW(
            wide_path.as_ptr(),
            std::ptr::null_mut(),
            0,
        )
    };

    if len == 0 {
        // Failed to get short path, return original
        return path.to_string_lossy().to_string();
    }

    // Allocate buffer and get the short path
    let mut buffer: Vec<u16> = vec![0; len as usize];
    let result = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetShortPathNameW(
            wide_path.as_ptr(),
            buffer.as_mut_ptr(),
            len,
        )
    };

    if result == 0 || result > len {
        // Failed, return original path
        return path.to_string_lossy().to_string();
    }

    // Convert back to String (trim null terminator)
    String::from_utf16_lossy(&buffer[..result as usize])
}

/// Convert a path to its short (8.3) form on Windows to avoid command line length limits.
/// On non-Windows platforms, returns the path string unchanged.
#[cfg(not(target_os = "windows"))]
pub fn to_short_path(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}
