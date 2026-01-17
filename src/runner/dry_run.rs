//! Dry-run mode for previewing what would happen.
//!
//! Shows configuration and the next feature without making any changes.

use anyhow::Result;

use crate::agent;
use crate::config::{Config, FEATURES_PATH, PROGRESS_PATH};
use crate::features::FeatureRepository;
use crate::log::ColoredOutput;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run in dry-run mode: show what would happen without making changes.
pub fn run_dry(
    iterations: u32,
    use_color: bool,
    sleep: u64,
    max_retries: u32,
    notify: bool,
) -> Result<()> {
    let config = Config::new(iterations, sleep, max_retries, notify)?;
    let repo = FeatureRepository::new(&config.features_path);

    // Show what feature would be worked on (without modifying the file)
    let current_feature = repo.get_current_feature().ok().flatten();
    let feature_name = if let Some(name) = current_feature {
        format!("{} (in_progress)", name)
    } else if let Some(name) = repo.peek_next_pending().ok().flatten() {
        format!("{} (pending)", name)
    } else {
        "(no pending features)".to_string()
    };

    // Use placeholder iteration_id for dry-run (actual ID is generated at runtime)
    let dry_run_iteration_id = "<iteration-id-generated-at-runtime>";
    let prompt = agent::build_prompt(
        FEATURES_PATH,
        PROGRESS_PATH,
        &feature_name,
        dry_run_iteration_id,
    );
    let out = ColoredOutput::new(use_color);

    println!("{}\n", out.info("=== Dry Run ==="));
    println!("Version:      {VERSION}");
    println!("Workdir:      {}", config.root_dir.display());
    println!("Features:     {}", config.features_path.display());
    println!("Progress:     {}", config.progress_path.display());
    println!("Iterations:   {iterations}");
    println!("Claude bin:   {}", config.claude_bin);
    println!("Next feature: {feature_name}");
    println!("\n{}\n{prompt}", out.info("=== Prompt ==="));

    Ok(())
}
