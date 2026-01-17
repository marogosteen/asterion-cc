//! Agent execution logic.
//!
//! Handles spawning and communicating with the Claude Code process.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::{AgentError, AgentResult};
use crate::progress_display::ProgressDisplay;
use crate::stream_parser;

/// Run the Claude agent with the given prompt.
///
/// Returns `Ok(true)` if the agent succeeded, `Ok(false)` if it failed or was interrupted.
pub fn run(
    prompt: &str,
    claude_bin: &str,
    interrupted: &Arc<AtomicBool>,
    use_color: bool,
) -> AgentResult<bool> {
    let mut display = ProgressDisplay::new(use_color);

    let mut child = Command::new(claude_bin)
        .args([
            "--dangerously-skip-permissions",
            "--output-format",
            "stream-json",
            "-p",
            prompt,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|source| AgentError::SpawnError {
            binary: claude_bin.to_string(),
            source,
        })?;

    let stdout = child.stdout.take().expect("stdout was piped");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        // Check for interrupt
        if interrupted.load(Ordering::SeqCst) {
            let _ = child.kill();
            return Ok(false);
        }

        let line = line.map_err(AgentError::ReadError)?;

        // Parse the stream event and update display
        let event = stream_parser::parse_line(&line);
        display.handle_event(&event);
    }

    // Check for interrupt before waiting
    if interrupted.load(Ordering::SeqCst) {
        let _ = child.kill();
        return Ok(false);
    }

    let status = child.wait().map_err(AgentError::ReadError)?;

    // Finish display if we didn't get a result event
    display.finish();

    Ok(status.success())
}
