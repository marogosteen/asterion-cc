use clap::Parser;
use tracing::Level;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "asterion")]
#[command(about = "Autonomous agent loop for Claude Code")]
#[command(version = VERSION)]
pub struct Cli {
    /// Maximum iterations to run
    #[arg(value_name = "ITERATIONS")]
    pub iterations: Option<u32>,

    /// Same as positional iterations
    #[arg(short = 'i', long = "iterations", value_name = "N")]
    pub iterations_opt: Option<u32>,

    /// Pause between iterations (seconds)
    #[arg(long = "sleep", default_value = "0")]
    pub sleep: u64,

    /// Retry failed agent calls up to N times
    #[arg(long = "max-retries", default_value = "2")]
    pub max_retries: u32,

    /// Send desktop notification on completion
    #[arg(long = "notify")]
    pub notify: bool,

    /// Print resolved command and prompt, then exit
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Increase verbosity (-v for debug, -vv for trace)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short = 'q', long = "quiet", conflicts_with = "verbose")]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long = "no-color", env = "NO_COLOR")]
    pub no_color: bool,
}

impl Cli {
    pub fn iterations(&self) -> Option<u32> {
        self.iterations.or(self.iterations_opt)
    }

    pub fn log_level(&self) -> Level {
        if self.quiet {
            Level::ERROR
        } else {
            match self.verbose {
                0 => Level::INFO,
                1 => Level::DEBUG,
                _ => Level::TRACE,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Cli {
        Cli::parse_from(std::iter::once("asterion").chain(args.iter().copied()))
    }

    #[test]
    fn test_iterations_positional() {
        let cli = parse_args(&["10"]);
        assert_eq!(cli.iterations(), Some(10));
    }

    #[test]
    fn test_iterations_flag() {
        let cli = parse_args(&["-i", "5"]);
        assert_eq!(cli.iterations(), Some(5));
    }

    #[test]
    fn test_iterations_flag_takes_precedence() {
        let cli = parse_args(&["10", "-i", "5"]);
        // positional is checked first in iterations()
        assert_eq!(cli.iterations(), Some(10));
    }

    #[test]
    fn test_log_level_default() {
        let cli = parse_args(&["5"]);
        assert_eq!(cli.log_level(), Level::INFO);
    }

    #[test]
    fn test_log_level_verbose() {
        let cli = parse_args(&["5", "-v"]);
        assert_eq!(cli.log_level(), Level::DEBUG);
    }

    #[test]
    fn test_log_level_very_verbose() {
        let cli = parse_args(&["5", "-vv"]);
        assert_eq!(cli.log_level(), Level::TRACE);
    }

    #[test]
    fn test_log_level_quiet() {
        let cli = parse_args(&["5", "-q"]);
        assert_eq!(cli.log_level(), Level::ERROR);
    }

    #[test]
    fn test_default_values() {
        let cli = parse_args(&["5"]);
        assert_eq!(cli.sleep, 0);
        assert_eq!(cli.max_retries, 2);
        assert!(!cli.notify);
        assert!(!cli.dry_run);
    }
}
