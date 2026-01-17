//! Main orchestration logic for the iteration loop.
//!
//! Contains the core loop that drives feature implementation iterations.

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::agent;
use crate::config::{Config, FEATURES_PATH, PROGRESS_PATH};
use crate::error::{RunnerError, RunnerResult};
use crate::features::FeatureRepository;
use crate::log::ColoredOutput;
use crate::notification;

/// Run the main iteration loop.
///
/// This function orchestrates the feature implementation cycle:
/// 1. Find the next pending feature
/// 2. Mark it as in_progress
/// 3. Run the agent to implement it
/// 4. Mark it as completed on success
/// 5. Repeat until all features are done or iterations are exhausted
pub fn run_loop(config: &Config, interrupted: &Arc<AtomicBool>, use_color: bool) -> RunnerResult<()> {
    let out = ColoredOutput::new(use_color);
    let repo = FeatureRepository::new(&config.features_path);

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
        let iteration_start = repo
            .start_iteration()
            .map_err(RunnerError::Features)?;

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
        match run_iteration_with_retry(&prompt, config, interrupted, use_color)? {
            IterationResult::Success => {
                // Success: mark feature as completed
                if let Some(completed_name) = repo.complete_iteration().map_err(RunnerError::Features)? {
                    println!("{}", out.success(&format!("Completed: {}", completed_name)));
                }
            }
            IterationResult::Interrupted => {
                info!(
                    "Stopped at iteration {} (feature: {})",
                    i, start.feature_name
                );
                return Ok(());
            }
        }

        // Check if all features are completed
        if repo.all_completed().map_err(RunnerError::Features)? {
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

/// Result of a single iteration attempt.
enum IterationResult {
    /// Iteration completed successfully.
    Success,
    /// Iteration was interrupted by user.
    Interrupted,
}

/// Run a single iteration with retry logic.
///
/// Returns `Ok(Success)` if succeeded, `Ok(Interrupted)` if interrupted.
fn run_iteration_with_retry(
    prompt: &str,
    config: &Config,
    interrupted: &Arc<AtomicBool>,
    use_color: bool,
) -> RunnerResult<IterationResult> {
    let mut retry_count = 0;

    loop {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            return Ok(IterationResult::Interrupted);
        }

        if retry_count > 0 {
            warn!("Retry {}/{}...", retry_count, config.max_retries);
        }

        debug!("Invoking Claude Code...");

        match agent::run(prompt, &config.claude_bin, interrupted, use_color) {
            Ok(true) => return Ok(IterationResult::Success),
            Ok(false) if interrupted.load(Ordering::SeqCst) => {
                // Interrupted during agent run
                return Ok(IterationResult::Interrupted);
            }
            Ok(false) => {
                retry_count += 1;
                if retry_count > config.max_retries {
                    error!("Agent failed after {} retries", config.max_retries);
                    return Err(RunnerError::MaxRetriesExceeded {
                        retries: config.max_retries,
                    });
                }
                thread::sleep(Duration::from_secs(2));
            }
            Err(e) => {
                retry_count += 1;
                error!("Agent error: {e}");
                if retry_count > config.max_retries {
                    return Err(RunnerError::Agent(e));
                }
                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
