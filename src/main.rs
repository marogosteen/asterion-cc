mod agent;
mod cli;
mod config;
mod features;
mod log;
mod notification;

use anyhow::{bail, Result};
use clap::Parser;
use std::fs;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use cli::Cli;
use config::{Config, FEATURES_PATH, PROGRESS_PATH};
use log::ColoredOutput;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let use_color = log::should_use_color(cli.no_color);
    log::init(cli.log_level(), cli.no_color);

    let iterations = cli
        .iterations()
        .ok_or_else(|| anyhow::anyhow!("iterations is required"))?;
    if iterations < 1 {
        bail!("iterations must be >= 1");
    }

    if cli.dry_run {
        return run_dry(iterations, use_color, cli.sleep, cli.max_retries, cli.notify);
    }

    let config = Config::new(iterations, cli.sleep, cli.max_retries, cli.notify)?;

    // Set up signal handler
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
        eprintln!("\nInterrupted. Shutting down gracefully...");
    })?;

    run_loop(&config, &interrupted, use_color)
}

fn run_dry(
    iterations: u32,
    use_color: bool,
    sleep: u64,
    max_retries: u32,
    notify: bool,
) -> Result<()> {
    let config = Config::new(iterations, sleep, max_retries, notify)?;

    // Show what feature would be worked on (without modifying the file)
    let current_feature = features::get_current_feature(&config.features_path)?;
    let feature_name = if let Some(name) = current_feature {
        format!("{} (in_progress)", name)
    } else if let Some(name) = features::peek_next_pending(&config.features_path)? {
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

fn run_loop(config: &Config, interrupted: &Arc<AtomicBool>, use_color: bool) -> Result<()> {
    let out = ColoredOutput::new(use_color);

    // Ensure progress file exists
    if let Some(parent) = config.progress_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !config.progress_path.exists() {
        fs::write(&config.progress_path, "")?;
    }

    info!(
        "Starting autonomous loop: {} iteration(s)",
        config.iterations
    );
    debug!(
        "Claude: {} --dangerously-skip-permissions",
        config.claude_bin
    );
    debug!("Features: {}", FEATURES_PATH);
    debug!("Progress: {}", PROGRESS_PATH);

    for i in 1..=config.iterations {
        // Check for interrupt at the start of each iteration
        if interrupted.load(Ordering::SeqCst) {
            warn!("Interrupted before iteration {}", i);
            return Ok(());
        }

        println!();
        println!("=== {} ===", out.iteration(i, config.iterations));

        // Start iteration: mark next pending feature as in_progress
        let iteration_start = features::start_iteration(&config.features_path)?;

        let Some(start) = iteration_start else {
            // No more pending features
            println!(
                "{}",
                out.success("All features are already completed or in progress.")
            );
            return Ok(());
        };

        info!("Working on: {} ({})", start.feature_name, start.iteration_id);

        // Build prompt with the specific feature name and iteration ID
        let prompt = agent::build_prompt(
            FEATURES_PATH,
            PROGRESS_PATH,
            &start.feature_name,
            &start.iteration_id,
        );

        // Run the agent
        match run_iteration_with_retry(&prompt, config, interrupted, use_color) {
            Ok(true) => {
                // Success: mark feature as completed
                if let Some(completed_name) =
                    features::complete_iteration(&config.features_path)?
                {
                    println!("{}", out.success(&format!("Completed: {}", completed_name)));
                }
            }
            Ok(false) => {
                // Interrupted
                info!(
                    "Stopped at iteration {} (feature: {})",
                    i, start.feature_name
                );
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        // Check if all features are completed
        if features::all_completed(&config.features_path)? {
            println!(
                "{}",
                out.success(&format!(
                    "All features complete after {} iteration(s).",
                    i
                ))
            );
            if config.notify {
                notification::send(&format!("Complete after {} iteration(s)", i));
            }
            return Ok(());
        }

        if config.sleep > 0 && i < config.iterations {
            info!("Sleeping {}s before next iteration...", config.sleep);
            thread::sleep(Duration::from_secs(config.sleep));
        }
    }

    info!(
        "Completed {} iteration(s). Features may still be pending.",
        config.iterations
    );
    if config.notify {
        notification::send(&format!("Finished {} iteration(s)", config.iterations));
    }

    Ok(())
}

/// Returns Ok(true) if succeeded, Ok(false) if interrupted, Err on failure
fn run_iteration_with_retry(
    prompt: &str,
    config: &Config,
    interrupted: &Arc<AtomicBool>,
    use_color: bool,
) -> Result<bool> {
    let mut retry_count = 0;

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            return Ok(false);
        }

        if retry_count > 0 {
            warn!("Retry {}/{}...", retry_count, config.max_retries);
        }

        debug!("Invoking Claude Code...");

        match agent::run(prompt, &config.claude_bin, interrupted, use_color) {
            Ok(true) => return Ok(true),
            Ok(false) if interrupted.load(Ordering::SeqCst) => {
                // Interrupted during agent run
                return Ok(false);
            }
            Ok(false) => {
                retry_count += 1;
                if retry_count > config.max_retries {
                    error!("Agent failed after {} retries", config.max_retries);
                    bail!("agent failed after {} retries", config.max_retries);
                }
                thread::sleep(Duration::from_secs(2));
            }
            Err(e) => {
                retry_count += 1;
                error!("Agent error: {e}");
                if retry_count > config.max_retries {
                    return Err(e);
                }
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
