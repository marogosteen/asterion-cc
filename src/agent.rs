use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Build the prompt for the agent
///
/// The feature_name is the specific feature to implement in this iteration.
/// The iteration_id uniquely identifies this iteration for feature lookup.
pub fn build_prompt(
    features_ref: &str,
    progress_ref: &str,
    feature_name: &str,
    iteration_id: &str,
) -> String {
    format!(
        r#"Read the following files:
- {features_ref}
- {progress_ref}

You are an autonomous software engineer. Your task is to implement a single feature.

## Current Task

Implement the feature: **{feature_name}**
Iteration ID: `{iteration_id}`

Find this feature in {features_ref} by matching the `iteration_id` field.

## Workflow

1. Read the features file and locate the feature with `iteration_id: "{iteration_id}"`.
2. Run `./.asterion/init.sh` to verify the project is in a clean state before changes.
3. Implement ONLY the specified feature following any project-specific instructions.
4. Run `./.asterion/init.sh` again to verify build, tests, and lint pass.
5. Append a concise progress note to {progress_ref}.
6. If this is a git repository, commit your changes.
7. Exit immediately after committing.

## Constraints

- Implement ONLY the feature with iteration_id "{iteration_id}". Do not work on other features.
- Stop after completing this single feature. Do not proceed to the next feature.
- Do NOT modify {features_ref} (status is managed externally).
- Follow the project structure defined in the features file.
- Do not modify unrelated code.
- Do not print, log, or commit secrets or credentials.
- Do not run destructive commands."#
    )
}

/// Create a spinner for long-running operations
pub fn create_spinner(message: &str, use_color: bool) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();

    let style = if use_color {
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    } else {
        ProgressStyle::default_spinner()
            .tick_chars("|/-\\")
            .template("{spinner} {msg}")
            .unwrap()
    };

    spinner.set_style(style);
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Run the Claude agent with the given prompt
///
/// Returns `Ok(true)` if the agent succeeded, `Ok(false)` if it failed or was interrupted
pub fn run(
    prompt: &str,
    claude_bin: &str,
    interrupted: &Arc<AtomicBool>,
    use_color: bool,
) -> Result<bool> {
    let spinner = create_spinner("Claude Code is working...", use_color);

    let mut child = Command::new(claude_bin)
        .args(["--dangerously-skip-permissions", "-p", prompt])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to spawn {}", claude_bin))?;

    let stdout = child.stdout.take().expect("stdout was piped");
    let reader = BufReader::new(stdout);

    spinner.finish_and_clear();

    for line in reader.lines() {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            let _ = child.kill();
            return Ok(false);
        }

        let line = line?;
        println!("{}", line);
    }

    // Check for interrupt before waiting
    if interrupted.load(Ordering::SeqCst) {
        let _ = child.kill();
        return Ok(false);
    }

    let status = child.wait()?;
    Ok(status.success())
}
