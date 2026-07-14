#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub app_name: String,
    pub window_title: String,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            app_name: "Unknown".to_string(),
            window_title: String::new(),
        }
    }
}

pub fn get_active_window() -> WindowInfo {
    #[cfg(target_os = "linux")]
    {
        return get_active_window_linux();
    }

    #[cfg(target_os = "macos")]
    {
        return get_active_window_macos();
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        WindowInfo::default()
    }
}

#[cfg(target_os = "linux")]
fn get_active_window_linux() -> WindowInfo {
    // Try xdotool first (X11)
    if let Ok(output) = std::process::Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
    {
        if output.status.success() {
            let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(id_output) = std::process::Command::new("xdotool")
                .arg("getactivewindow")
                .output()
            {
                if id_output.status.success() {
                    let wid = String::from_utf8_lossy(&id_output.stdout).trim().to_string();
                    if let Ok(prop_output) = std::process::Command::new("xprop")
                        .args(["-id", &wid, "WM_CLASS"])
                        .output()
                    {
                        if prop_output.status.success() {
                            let prop = String::from_utf8_lossy(&prop_output.stdout);
                            if let Some(class) = prop.split('"').nth(3) {
                                return WindowInfo {
                                    app_name: class.to_string(),
                                    window_title: title,
                                };
                            }
                        }
                    }
                }
            }
            return WindowInfo {
                app_name: extract_app_from_title(&title),
                window_title: title,
            };
        }
    }

    // Fallback: try hyprctl (Wayland/Hyprland)
    if let Ok(output) = std::process::Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
    {
        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            let app = extract_json_string(&json_str, "class");
            let title = extract_json_string(&json_str, "title");
            if !app.is_empty() {
                return WindowInfo {
                    app_name: app,
                    window_title: title,
                };
            }
        }
    }

    WindowInfo::default()
}

#[cfg(target_os = "macos")]
fn get_active_window_macos() -> WindowInfo {
    let script = r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set appName to name of frontApp
            set winTitle to ""
            try
                set winTitle to name of front window of frontApp
            end try
            return appName & "|||" & winTitle
        end tell
    "#;

    if let Ok(output) = std::process::Command::new("osascript")
        .args(["-e", script])
        .output()
    {
        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let parts: Vec<&str> = result.splitn(2, "|||").collect();
            return WindowInfo {
                app_name: parts.first().unwrap_or(&"Unknown").to_string(),
                window_title: parts.get(1).unwrap_or(&"").to_string(),
            };
        }
    }

    WindowInfo::default()
}

fn extract_app_from_title(title: &str) -> String {
    title
        .rsplit(" - ")
        .next()
        .unwrap_or("Unknown")
        .trim()
        .to_string()
}

#[allow(dead_code)]
fn extract_json_string(json: &str, key: &str) -> String {
    let search = format!("\"{}\"", key);
    if let Some(pos) = json.find(&search) {
        let after = &json[pos + search.len()..];
        if let Some(colon) = after.find(':') {
            let after_colon = &after[colon + 1..];
            if let Some(open) = after_colon.find('"') {
                let after_open = &after_colon[open + 1..];
                if let Some(close) = after_open.find('"') {
                    return after_open[..close].to_string();
                }
            }
        }
    }
    String::new()
}
