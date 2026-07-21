#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub app_name: String,
    pub window_title: String,
}

const MAX_WINDOW_FIELD_LENGTH: usize = 512;

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
    return get_active_window_linux();

    #[cfg(target_os = "macos")]
    return get_active_window_macos();

    #[cfg(target_os = "windows")]
    return get_active_window_windows();

    #[allow(unreachable_code)]
    WindowInfo::default()
}

#[cfg(target_os = "linux")]
fn get_active_window_linux() -> WindowInfo {
    // X11 and XWayland. This also works under GNOME Wayland for XWayland apps.
    if let Some(window) = x11_active_window() {
        return window;
    }
    // Sway's IPC is the supported Wayland path for Sway-based desktops.
    if let Some(window) = sway_active_window() {
        return window;
    }
    // Hyprland exposes the active client through its documented CLI.
    if let Some(window) = hyprland_active_window() {
        return window;
    }
    // GNOME does not offer a stable public active-window API on Wayland. Shell
    // Eval is best-effort and often disabled by distro policy; failure remains
    // a safe Unknown fallback rather than blocking capture.
    if let Some(window) = gnome_shell_active_window() {
        return window;
    }
    // Optional X11 utility fallback. AppImage users do not need this package;
    // it only improves attribution on systems where the probes above fail.
    if let Some(window) = xdotool_active_window() {
        return window;
    }
    WindowInfo::default()
}

#[cfg(target_os = "linux")]
fn x11_active_window() -> Option<WindowInfo> {
    let active = std::process::Command::new("xprop")
        .args(["-root", "_NET_ACTIVE_WINDOW"])
        .output()
        .ok()?;
    if !active.status.success() {
        return None;
    }
    let window_id = parse_active_window_id(&String::from_utf8_lossy(&active.stdout))?;
    let properties = std::process::Command::new("xprop")
        .args(["-id", &window_id, "WM_CLASS", "_NET_WM_NAME", "WM_NAME"])
        .output()
        .ok()?;
    if !properties.status.success() {
        return None;
    }
    parse_xprop_window(&String::from_utf8_lossy(&properties.stdout))
}

#[cfg(target_os = "linux")]
fn sway_active_window() -> Option<WindowInfo> {
    let output = std::process::Command::new("swaymsg")
        .args(["-t", "get_tree", "-r"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let tree: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    find_sway_focused_node(&tree)
}

#[cfg(target_os = "linux")]
fn find_sway_focused_node(node: &serde_json::Value) -> Option<WindowInfo> {
    if node.get("focused").and_then(serde_json::Value::as_bool) == Some(true) {
        let app_name = node
            .get("app_id")
            .and_then(serde_json::Value::as_str)
            .or_else(|| {
                node.get("window_properties")
                    .and_then(|value| value.get("class"))
                    .and_then(serde_json::Value::as_str)
            })
            .unwrap_or_default();
        let title = node
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        if !app_name.is_empty() {
            return Some(WindowInfo {
                app_name: app_name.to_string(),
                window_title: title.to_string(),
            });
        }
    }
    for key in ["nodes", "floating_nodes"] {
        if let Some(children) = node.get(key).and_then(serde_json::Value::as_array) {
            for child in children {
                if let Some(found) = find_sway_focused_node(child) {
                    return Some(found);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn hyprland_active_window() -> Option<WindowInfo> {
    let output = std::process::Command::new("hyprctl")
        .args(["activewindow", "-j"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let window: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let app_name = window.get("class")?.as_str()?.trim();
    if app_name.is_empty() {
        return None;
    }
    Some(WindowInfo {
        app_name: app_name.to_string(),
        window_title: window
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

#[cfg(target_os = "linux")]
fn gnome_shell_active_window() -> Option<WindowInfo> {
    if !std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("gnome")
    {
        return None;
    }
    let script = "let w = global.display.focus_window; JSON.stringify({app: w ? w.get_wm_class() : '', title: w ? w.get_title() : ''});";
    let output = std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/Shell",
            "--method",
            "org.gnome.Shell.Eval",
            script,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let result = String::from_utf8_lossy(&output.stdout).replace("\\\"", "\"");
    let app_name = extract_json_string(&result, "app");
    if app_name.is_empty() {
        return None;
    }
    Some(WindowInfo {
        app_name,
        window_title: extract_json_string(&result, "title"),
    })
}

#[cfg(target_os = "linux")]
fn xdotool_active_window() -> Option<WindowInfo> {
    let id = std::process::Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
        .ok()?;
    if !id.status.success() {
        return None;
    }
    let id = String::from_utf8_lossy(&id.stdout).trim().to_string();
    if id.is_empty() {
        return None;
    }
    let title = std::process::Command::new("xdotool")
        .args(["getwindowname", &id])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default();
    let pid = std::process::Command::new("xdotool")
        .args(["getwindowpid", &id])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<u32>()
                .ok()
        });
    if let Some(pid) = pid {
        if let Some(app_name) = std::fs::read_to_string(format!("/proc/{pid}/comm"))
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return parse_xdotool_window(&app_name, &title);
        }
    }

    // Some desktop-owned X11 windows have no PID. WM_CLASS still gives us a
    // truthful application identity without inventing one from the title.
    std::process::Command::new("xprop")
        .args(["-id", &id, "WM_CLASS", "_NET_WM_NAME", "WM_NAME"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| parse_xprop_window(&String::from_utf8_lossy(&output.stdout)))
        .or_else(|| parse_xdotool_window("X11 window", &title))
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
            if let Some(window) = parse_delimited_window(&String::from_utf8_lossy(&output.stdout)) {
                return window;
            }
        }
    }
    WindowInfo::default()
}

#[cfg(target_os = "windows")]
fn get_active_window_windows() -> WindowInfo {
    let script = r#"
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class MnemoWindow {
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
}
'@;
$window = [MnemoWindow]::GetForegroundWindow();
$pid = 0; [MnemoWindow]::GetWindowThreadProcessId($window, [ref]$pid) | Out-Null;
$title = New-Object System.Text.StringBuilder 1024; [MnemoWindow]::GetWindowText($window, $title, $title.Capacity) | Out-Null;
$process = Get-Process -Id $pid -ErrorAction SilentlyContinue;
"$($process.ProcessName)|||$($title.ToString())"
"#;
    if let Ok(output) = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
    {
        if output.status.success() {
            if let Some(window) = parse_delimited_window(&String::from_utf8_lossy(&output.stdout)) {
                if window.app_name != "Unknown" {
                    return window;
                }
            }
        }
    }
    WindowInfo::default()
}

fn parse_active_window_id(value: &str) -> Option<String> {
    value
        .split_whitespace()
        .find(|part| part.starts_with("0x") && *part != "0x0")
        .map(str::to_string)
}

fn parse_xprop_window(value: &str) -> Option<WindowInfo> {
    let app_name = value
        .lines()
        .find(|line| line.starts_with("WM_CLASS"))
        .and_then(|line| line.rsplit('"').nth(1))?
        .trim();
    if app_name.is_empty() {
        return None;
    }
    let window_title = value
        .lines()
        .find(|line| line.starts_with("_NET_WM_NAME") || line.starts_with("WM_NAME"))
        .and_then(|line| line.split('"').nth(1))
        .unwrap_or_default()
        .to_string();
    parse_window_fields(app_name, &window_title)
}

#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn parse_delimited_window(value: &str) -> Option<WindowInfo> {
    let (app_name, window_title) = value.trim().split_once("|||")?;
    parse_window_fields(app_name, window_title)
}

fn parse_window_fields(app_name: &str, window_title: &str) -> Option<WindowInfo> {
    let app_name = sanitize_field(app_name);
    if app_name.is_empty() || app_name == "Unknown" {
        return None;
    }
    Some(WindowInfo {
        app_name,
        window_title: sanitize_field(window_title),
    })
}

fn sanitize_field(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_WINDOW_FIELD_LENGTH)
        .collect()
}

fn extract_json_string(json: &str, key: &str) -> String {
    let search = format!("\"{key}\"");
    let Some(pos) = json.find(&search) else {
        return String::new();
    };
    let after = &json[pos + search.len()..];
    let Some(colon) = after.find(':') else {
        return String::new();
    };
    let after_colon = &after[colon + 1..];
    let Some(open) = after_colon.find('"') else {
        return String::new();
    };
    let after_open = &after_colon[open + 1..];
    let Some(close) = after_open.find('"') else {
        return String::new();
    };
    after_open[..close].to_string()
}

#[cfg(target_os = "linux")]
fn parse_xdotool_window(app_name: &str, window_title: &str) -> Option<WindowInfo> {
    parse_window_fields(app_name, window_title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_x11_active_window_and_properties() {
        assert_eq!(
            parse_active_window_id("_NET_ACTIVE_WINDOW(WINDOW): window id # 0x4a00007"),
            Some("0x4a00007".into())
        );
        assert_eq!(
            parse_active_window_id("_NET_ACTIVE_WINDOW(WINDOW): window id # 0x0"),
            None
        );
        assert_eq!(parse_xprop_window("WM_CLASS(STRING) = \"code\", \"Code\"\n_NET_WM_NAME(UTF8_STRING) = \"lib.rs - Mnemo\""), Some(WindowInfo { app_name: "Code".into(), window_title: "lib.rs - Mnemo".into() }));
    }

    #[test]
    fn parses_gnome_eval_json() {
        assert_eq!(
            extract_json_string(
                "(true, '{\\\"app\\\":\\\"org.gnome.Terminal\\\",\\\"title\\\":\\\"Terminal\\\"}')"
                    .replace("\\\"", "\"")
                    .as_str(),
                "app"
            ),
            "org.gnome.Terminal"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_optional_xdotool_metadata() {
        assert_eq!(
            parse_xdotool_window("firefox", "Mnemo - Mozilla Firefox"),
            Some(WindowInfo {
                app_name: "firefox".into(),
                window_title: "Mnemo - Mozilla Firefox".into(),
            })
        );
        assert_eq!(parse_xdotool_window("", "Terminal"), None);
    }

    #[test]
    fn bounds_and_sanitizes_provider_output() {
        let long_name = format!("  editor\n{}", "x".repeat(600));
        let window = parse_delimited_window(&format!("{long_name}||| title\r\n")).unwrap();
        assert_eq!(window.app_name.chars().count(), MAX_WINDOW_FIELD_LENGTH);
        assert!(!window.app_name.contains('\n'));
        assert_eq!(window.window_title, "title");
    }

    #[test]
    fn rejects_unknown_or_malformed_provider_output() {
        assert_eq!(parse_delimited_window("Unknown|||title"), None);
        assert_eq!(parse_delimited_window("editor"), None);
        assert_eq!(parse_window_fields("\n", "title"), None);
    }
}
