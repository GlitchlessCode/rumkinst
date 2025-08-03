use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use rumkinst::config::identifier::Identifier;

#[derive(Debug, Parser)]
#[command(version, about, author, long_about = None)]
pub struct Rumkinst {
    /// When to use terminal color
    #[arg(global = true, value_enum, long, default_value = "auto")]
    pub color: ColorDisplay,

    /// What log level to use. Can also be set using environment variables
    #[arg(global = true, value_enum, long, default_value = "info")]
    pub log_level: LogLevel,

    #[command(subcommand)]
    pub subcommand: Command,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorDisplay {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Make all installer artifacts
    Make {
        /// Path to rumkinst.toml
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Create a new rumkinst directory, with some defaults
    New {
        /// Name of the package
        name: Identifier,

        /// Name of rumkinst
        #[arg(long, default_value = "rumkinst")]
        dir_name: Identifier,
    },
}
