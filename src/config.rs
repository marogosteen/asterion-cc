use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Fixed paths for asterion files
pub const FEATURES_PATH: &str = ".asterion/features.json";
pub const PROGRESS_PATH: &str = ".asterion/progress.md";

/// Resolved runtime configuration
pub struct Config {
    pub root_dir: PathBuf,
    pub features_path: PathBuf,
    pub progress_path: PathBuf,
    pub iterations: u32,
    pub sleep: u64,
    pub max_retries: u32,
    pub notify: bool,
    pub claude_bin: String,
}

impl Config {
    pub fn new(iterations: u32, sleep: u64, max_retries: u32, notify: bool) -> Result<Self> {
        let root_dir = std::env::current_dir().context("failed to get current directory")?;

        let features_path = root_dir.join(FEATURES_PATH);
        let progress_path = root_dir.join(PROGRESS_PATH);

        if !features_path.exists() {
            bail!(
                "features file not found: {}",
                features_path.display()
            );
        }

        let claude_bin = find_claude_binary();

        Ok(Self {
            root_dir,
            features_path,
            progress_path,
            iterations,
            sleep,
            max_retries,
            notify,
            claude_bin,
        })
    }
}

fn find_claude_binary() -> String {
    if which::which("claude").is_ok() {
        return "claude".to_string();
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let paths = [
        format!("{}/.claude/local/claude", home),
        "/usr/local/bin/claude".to_string(),
    ];

    for p in &paths {
        if Path::new(p).is_file() {
            return p.clone();
        }
    }

    "claude".to_string()
}
