//! Agent module for interacting with Claude Code.
//!
//! This module handles prompt building and execution of the Claude Code CLI.

mod executor;
mod prompt;

pub use executor::run;
pub use prompt::build_prompt;
