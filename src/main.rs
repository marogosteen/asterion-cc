//! Asterion - Autonomous agent loop for Claude Code.
//!
//! This CLI tool orchestrates multiple iterations of Claude Code
//! to implement features defined in a features.json file.

mod agent;
mod cli;
mod config;
mod error;
mod features;
mod log;
mod notification;
mod progress_display;
mod runner;
mod stream_parser;

use anyhow::{bail, Result};
use clap::Parser;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::error;

use cli::Cli;
use config::Config;

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
        .ok_or_else(|| anyhow::anyhow!("iterations parameter is required (use positional argument or --iterations)"))?;
    if iterations < 1 {
        bail!("iterations must be >= 1, got {}", iterations);
    }

    if cli.dry_run {
        return runner::run_dry(iterations, use_color, cli.sleep, cli.max_retries, cli.notify);
    }

    let config = Config::new(iterations, cli.sleep, cli.max_retries, cli.notify)?;

    // Set up signal handler
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);

    ctrlc::set_handler(move || {
        interrupted_clone.store(true, Ordering::SeqCst);
        eprintln!("\nInterrupted. Shutting down gracefully...");
    })?;

    runner::run_loop(&config, &interrupted, use_color).map_err(|e| {
        // Convert RunnerError to anyhow::Error for top-level handling
        anyhow::anyhow!("{}", e)
    })
}
