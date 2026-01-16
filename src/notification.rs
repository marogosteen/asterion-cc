use std::process::Command;

const TITLE: &str = "Asterion";

/// Send a desktop notification (macOS / Linux)
pub fn send(message: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!(r#"display notification "{message}" with title "{TITLE}""#);
        let _ = Command::new("osascript").args(["-e", &script]).output();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send").args([TITLE, message]).output();
    }
}
