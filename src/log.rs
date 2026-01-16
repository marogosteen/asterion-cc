use owo_colors::OwoColorize;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

/// Initialize the tracing subscriber with the given log level and color settings
pub fn init(level: Level, no_color: bool) {
    let filter = EnvFilter::from_default_env().add_directive(level.into());

    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_span_events(FmtSpan::NONE)
        .without_time();

    if no_color {
        builder.with_ansi(false).init();
    } else {
        builder.init();
    }
}

/// Check if colors should be enabled based on CLI flag and terminal support
pub fn should_use_color(no_color: bool) -> bool {
    if no_color {
        return false;
    }
    // Check if stdout is a terminal
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

/// Colored status messages
pub struct ColoredOutput {
    use_color: bool,
}

#[allow(dead_code)]
impl ColoredOutput {
    pub fn new(use_color: bool) -> Self {
        Self { use_color }
    }

    pub fn success(&self, msg: &str) -> String {
        if self.use_color {
            msg.green().bold().to_string()
        } else {
            msg.to_string()
        }
    }

    pub fn error(&self, msg: &str) -> String {
        if self.use_color {
            msg.red().bold().to_string()
        } else {
            msg.to_string()
        }
    }

    pub fn warn(&self, msg: &str) -> String {
        if self.use_color {
            msg.yellow().to_string()
        } else {
            msg.to_string()
        }
    }

    pub fn info(&self, msg: &str) -> String {
        if self.use_color {
            msg.cyan().to_string()
        } else {
            msg.to_string()
        }
    }

    pub fn dim(&self, msg: &str) -> String {
        if self.use_color {
            msg.dimmed().to_string()
        } else {
            msg.to_string()
        }
    }

    pub fn iteration(&self, current: u32, total: u32) -> String {
        let msg = format!("Iteration {current}/{total}");
        if self.use_color {
            msg.blue().bold().to_string()
        } else {
            msg
        }
    }
}
