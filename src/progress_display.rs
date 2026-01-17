//! Progress display for Claude Code execution.
//!
//! Shows real-time tool usage and elapsed time during agent execution.

use crate::stream_parser::StreamEvent;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::time::Instant;

/// Progress display for Claude Code execution.
///
/// Shows current tool usage with emoji icons and tracks elapsed time.
pub struct ProgressDisplay {
    spinner: ProgressBar,
    start_time: Instant,
    use_color: bool,
    finished: bool,
}

impl ProgressDisplay {
    /// Create a new progress display.
    pub fn new(use_color: bool) -> Self {
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
        spinner.set_message("Starting...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        Self {
            spinner,
            start_time: Instant::now(),
            use_color,
            finished: false,
        }
    }

    /// Handle a stream event and update the display.
    pub fn handle_event(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::ToolUse { tool_name } => {
                self.show_tool(tool_name);
            }
            StreamEvent::Text { content } => {
                self.show_text(content);
            }
            StreamEvent::Result { success } => {
                self.show_result(*success);
            }
            StreamEvent::Unknown => {
                // Ignore unknown events
            }
        }
    }

    /// Display a tool usage event.
    fn show_tool(&mut self, tool_name: &str) {
        let icon = tool_icon(tool_name);
        let elapsed = format_elapsed(self.start_time.elapsed());

        let message = if self.use_color {
            format!(
                "{} {} {}",
                icon,
                tool_name.cyan(),
                format!("[{}]", elapsed).dimmed()
            )
        } else {
            format!("{} {} [{}]", icon, tool_name, elapsed)
        };

        self.spinner.set_message(message);
    }

    /// Display text content from Claude.
    fn show_text(&mut self, content: &str) {
        // Suspend spinner, print text, resume spinner
        self.spinner.suspend(|| {
            print!("{}", content);
            let _ = io::stdout().flush();
        });
    }

    /// Display the result of execution.
    fn show_result(&mut self, success: bool) {
        if self.finished {
            return;
        }
        self.finished = true;

        let elapsed = format_elapsed(self.start_time.elapsed());

        if success {
            let message = if self.use_color {
                format!("{} {} [{}]", "✓", "Completed".green().bold(), elapsed)
            } else {
                format!("Done [{}]", elapsed)
            };
            self.spinner.finish_with_message(message);
        } else {
            let message = if self.use_color {
                format!("{} {} [{}]", "✗", "Failed".red().bold(), elapsed)
            } else {
                format!("Failed [{}]", elapsed)
            };
            self.spinner.finish_with_message(message);
        }
    }

    /// Finish the progress display and show elapsed time.
    ///
    /// This is a no-op if the display was already finished by a Result event.
    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;

        let elapsed = format_elapsed(self.start_time.elapsed());
        let message = if self.use_color {
            format!("{} [{}]", "Done".green(), elapsed)
        } else {
            format!("Done [{}]", elapsed)
        };
        self.spinner.finish_with_message(message);
    }
}

/// Get an emoji icon for a tool name.
fn tool_icon(tool_name: &str) -> &'static str {
    match tool_name {
        "Read" | "NotebookRead" => "📖",
        "Write" | "NotebookEdit" => "💾",
        "Edit" => "✏️",
        "Bash" => "🔧",
        "Grep" => "🔍",
        "Glob" => "📂",
        "WebSearch" => "🌐",
        "WebFetch" => "🌐",
        "Task" => "🤖",
        "TodoWrite" => "📝",
        "AskUserQuestion" => "❓",
        _ => "⚙️",
    }
}

/// Format a duration as a human-readable string.
fn format_elapsed(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_icon() {
        assert_eq!(tool_icon("Read"), "📖");
        assert_eq!(tool_icon("Bash"), "🔧");
        assert_eq!(tool_icon("Edit"), "✏️");
        assert_eq!(tool_icon("UnknownTool"), "⚙️");
    }

    #[test]
    fn test_format_elapsed() {
        assert_eq!(format_elapsed(std::time::Duration::from_secs(5)), "5s");
        assert_eq!(format_elapsed(std::time::Duration::from_secs(65)), "1m 5s");
        assert_eq!(
            format_elapsed(std::time::Duration::from_secs(125)),
            "2m 5s"
        );
    }
}
