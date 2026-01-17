//! Custom error types for asterion.
//!
//! Provides domain-specific error types for better error handling and user feedback.

use std::path::PathBuf;
use thiserror::Error;

/// Errors related to configuration and setup.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ConfigError {
    #[error("features file not found: {0}")]
    FeaturesFileNotFound(PathBuf),

    #[error("failed to get current directory: {0}")]
    CurrentDirError(#[source] std::io::Error),

    #[error("iterations parameter is required (use positional argument or --iterations)")]
    IterationsRequired,

    #[error("iterations must be >= 1, got {0}")]
    InvalidIterations(u32),
}

/// Errors related to features file operations.
#[derive(Error, Debug)]
pub enum FeaturesError {
    #[error("failed to read features file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write features file '{path}': {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse features file '{path}': {source}")]
    ParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("invalid features file format: must be array or object with 'features' key")]
    #[allow(dead_code)]
    InvalidFormat,

    #[error("no pending features available")]
    #[allow(dead_code)]
    NoPendingFeatures,

    #[error("no in-progress feature found")]
    #[allow(dead_code)]
    NoInProgressFeature,
}

/// Errors related to agent execution.
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("failed to spawn claude binary '{binary}': {source}. Is it installed and in PATH?")]
    SpawnError {
        binary: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read agent output: {0}")]
    ReadError(#[source] std::io::Error),

    #[error("agent execution was interrupted")]
    #[allow(dead_code)]
    Interrupted,
}

/// Errors related to the main runner loop.
#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("agent failed after {retries} retries. Check .asterion/init.sh and logs for details")]
    MaxRetriesExceeded { retries: u32 },

    #[error("runner was interrupted")]
    #[allow(dead_code)]
    Interrupted,

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Features(#[from] FeaturesError),

    #[error(transparent)]
    Agent(#[from] AgentError),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Result type alias using RunnerError.
pub type RunnerResult<T> = Result<T, RunnerError>;

/// Result type alias using FeaturesError.
pub type FeaturesResult<T> = Result<T, FeaturesError>;

/// Result type alias using AgentError.
pub type AgentResult<T> = Result<T, AgentError>;

/// Result type alias using ConfigError.
#[allow(unused)]
pub type ConfigResult<T> = Result<T, ConfigError>;
