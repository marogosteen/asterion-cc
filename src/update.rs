//! Self-update functionality.
//!
//! Checks for updates and runs the install script to update.

use anyhow::{bail, Context, Result};
use std::io::{self, Write};
use std::process::Command;

const REPO: &str = "marogosteen/asterion-cc";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check for updates and install if available.
pub fn run_update() -> Result<()> {
    let current = VERSION;
    let latest = fetch_latest_version()?;

    println!("Current: v{current}");
    println!("Latest:  {latest}");

    // Compare versions (strip 'v' prefix for comparison)
    let latest_clean = latest.trim_start_matches('v');
    if current == latest_clean {
        println!("\nAlready up to date.");
        return Ok(());
    }

    // Check if latest is actually newer
    if !is_newer_version(latest_clean, current) {
        println!("\nYou are running a newer version than the latest release.");
        return Ok(());
    }

    println!();
    print!("Update available. Proceed? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Update cancelled.");
        return Ok(());
    }

    println!("\nUpdating...");
    run_install_script()?;

    println!("\nUpdate complete. Please restart asterion.");
    Ok(())
}

/// Fetch the latest release version from GitHub.
fn fetch_latest_version() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");

    let response: serde_json::Value = ureq::get(&url)
        .header("User-Agent", "asterion-updater")
        .call()
        .context("failed to fetch latest release")?
        .body_mut()
        .read_json()
        .context("failed to parse release info")?;

    let tag = response["tag_name"]
        .as_str()
        .context("missing tag_name in response")?;

    Ok(tag.to_string())
}

/// Run the install script to perform the actual update.
fn run_install_script() -> Result<()> {
    let install_url = format!(
        "https://raw.githubusercontent.com/{REPO}/main/install.sh"
    );

    let status = Command::new("sh")
        .args(["-c", &format!("curl -fsSL {install_url} | sh")])
        .status()
        .context("failed to run install script")?;

    if !status.success() {
        bail!("install script failed");
    }

    Ok(())
}

/// Check if `latest` is newer than `current` using simple semver comparison.
fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };

    match (parse(latest), parse(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("2.1.0", "2.0.0"));
        assert!(is_newer_version("2.0.1", "2.0.0"));
        assert!(is_newer_version("3.0.0", "2.9.9"));
        assert!(!is_newer_version("2.0.0", "2.0.0"));
        assert!(!is_newer_version("2.0.0", "2.1.0"));
        assert!(!is_newer_version("1.9.9", "2.0.0"));
    }
}
