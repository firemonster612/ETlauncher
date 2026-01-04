// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Check if the system is using dark mode via freedesktop portal (modern standard)
/// Returns: 0 = no preference, 1 = prefer dark, 2 = prefer light
#[cfg(target_os = "linux")]
fn get_freedesktop_color_scheme() -> Option<u32> {
    // Query the freedesktop portal for color-scheme preference
    // This is the modern standard used by GNOME 42+, KDE Plasma 5.24+, etc.
    let output = std::process::Command::new("dbus-send")
        .args([
            "--session",
            "--print-reply=literal",
            "--dest=org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings.Read",
            "string:org.freedesktop.appearance",
            "string:color-scheme",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Parse the output to extract the uint32 value
    // Output format: "   variant       variant          uint32 1"
    let stdout = String::from_utf8_lossy(&output.stdout);
    for word in stdout.split_whitespace().rev() {
        if let Ok(value) = word.parse::<u32>() {
            return Some(value);
        }
    }

    None
}

/// Check if the system is using dark mode
#[cfg(target_os = "linux")]
fn is_dark_mode() -> bool {
    // First try the modern freedesktop portal (GNOME 42+, KDE Plasma 5.24+)
    if let Some(scheme) = get_freedesktop_color_scheme() {
        // 1 = prefer dark
        if scheme == 1 {
            return true;
        }
    }

    // Fallback: Check GNOME color scheme via gsettings
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if output.status.success() {
            let scheme = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if scheme.contains("dark") {
                return true;
            }
        }
    }

    // Fallback: Check KDE color scheme
    if let Ok(output) = std::process::Command::new("kreadconfig5")
        .args(["--group", "General", "--key", "ColorScheme"])
        .output()
    {
        if output.status.success() {
            let scheme = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if scheme.contains("dark") {
                return true;
            }
        }
    }

    // Fallback: check GTK theme name for dark indicators
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        if output.status.success() {
            let theme = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if theme.contains("dark") {
                return true;
            }
        }
    }

    false
}

/// Detect the GTK theme from GNOME or KDE settings
#[cfg(target_os = "linux")]
fn detect_gtk_theme() -> Option<String> {
    // Try GNOME settings first
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        if output.status.success() {
            let theme = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
            if !theme.is_empty() {
                return Some(theme);
            }
        }
    }

    // Try KDE/Plasma settings
    if let Ok(output) = std::process::Command::new("kreadconfig5")
        .args(["--group", "General", "--key", "widgetStyle"])
        .output()
    {
        if output.status.success() {
            let style = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !style.is_empty() {
                return Some(style);
            }
        }
    }

    None
}

fn main() {
    // On Linux, ensure GTK dialogs and elements follow the system theme
    #[cfg(target_os = "linux")]
    {
        // Use XDG Desktop Portal for file dialogs - integrates with desktop environment
        std::env::set_var("GTK_USE_PORTAL", "1");

        // Ensure WebKitGTK uses the correct theme settings
        // This helps with context menus, selection dialogs, etc.
        if std::env::var("GTK_THEME").is_err() {
            if let Some(mut theme) = detect_gtk_theme() {
                // If dark mode is enabled and theme doesn't already have :dark suffix,
                // try appending it for WebKitGTK context menus
                let is_dark = is_dark_mode();
                if is_dark && !theme.to_lowercase().contains("dark") && !theme.contains(":") {
                    // Try the dark variant
                    theme = format!("{}:dark", theme);
                }
                std::env::set_var("GTK_THEME", &theme);
            } else if is_dark_mode() {
                // No theme detected but dark mode is on - use Adwaita:dark as fallback
                std::env::set_var("GTK_THEME", "Adwaita:dark");
            }
        }
    }

    etlauncher_lib::run()
}
