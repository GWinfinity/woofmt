use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "woofmt",
    about = "An extremely fast Go linter and code formatter",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Files or directories to check (default: current directory).
    /// NOTE: no `default_value` here — with a default, clap would swallow
    /// the subcommand name (`woofmt fmt ...`) as a file path.
    #[arg(value_name = "FILES")]
    pub files: Vec<PathBuf>,

    /// Enable all lint rules
    #[arg(short, long, global = true)]
    pub all: bool,

    /// Disable specific rules (comma-separated)
    #[arg(short, long, value_delimiter = ',', global = true)]
    pub ignore: Vec<String>,

    /// Select specific rules (comma-separated)
    #[arg(short, long, value_delimiter = ',', global = true)]
    pub select: Vec<String>,

    /// Configuration file path
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Number of threads to use
    #[arg(short, long, global = true, value_name = "N")]
    pub threads: Option<usize>,

    /// Number of parallel workers (alias for threads)
    #[arg(long, global = true, value_name = "N")]
    pub parallel: Option<usize>,

    /// Show statistics
    #[arg(long, global = true)]
    pub statistics: bool,

    /// Quiet mode (suppress output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output format
    #[arg(long, global = true, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Lint Go source files
    Check {
        /// Files or directories to check
        #[arg(value_name = "FILES", default_value = ".")]
        files: Vec<PathBuf>,

        /// Apply auto-fixes where possible (safe fixes only by default)
        #[arg(long)]
        fix: bool,

        /// Also apply unsafe fixes (may change program semantics)
        #[arg(long)]
        unsafe_fixes: bool,

        /// Exit with error code if issues found
        #[arg(long)]
        exit_non_zero_on_fix: bool,
    },

    /// Format Go source files
    #[command(alias = "fmt")]
    Format {
        /// Files or directories to format
        #[arg(value_name = "FILES", default_value = ".")]
        files: Vec<PathBuf>,

        /// Check if files are formatted without modifying them
        #[arg(long)]
        check: bool,

        /// Write formatted output to stdout instead of files
        #[arg(long)]
        stdout: bool,
    },

    /// Show all enabled lint rules
    Rules {
        /// Show all available rules, not just enabled ones
        #[arg(short, long)]
        all: bool,
    },

    /// Initialize a configuration file
    Init {
        /// Create configuration with stricter defaults
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output
    Json,
    /// GitHub Actions format
    Github,
}

impl Cli {
    pub fn get_files(&self) -> Vec<PathBuf> {
        match &self.command {
            Some(Commands::Check { files, .. }) => files.clone(),
            Some(Commands::Format { files, .. }) => files.clone(),
            _ => self.files.clone(),
        }
    }
}
