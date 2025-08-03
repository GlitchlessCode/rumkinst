mod cli;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use cli::{Command, Rumkinst};
use rumkinst::{
    config::{Config, find_config_file_at, identifier::Identifier},
    error_log::Log,
};

fn setup_logging(config: &Rumkinst) {
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or(
                config
                    .log_level
                    .to_possible_value()
                    .expect("log level possible value should never be None")
                    .get_name(),
            )
            .default_write_style_or(
                config
                    .color
                    .to_possible_value()
                    .expect("color display possible value should never be None")
                    .get_name(),
            ),
    )
    .init();
}

fn move_to_config_parent(path: &Path) -> Result<()> {
    log::trace!("Moving working directory");
    std::env::set_current_dir(path.parent().context("could not find parent directory")?)
        .context("failed to change working directory")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rumkinst = Rumkinst::parse();

    setup_logging(&rumkinst);

    match rumkinst.subcommand {
        Command::New { name, dir_name } => {
            command_new(name, PathBuf::from(format!("./{}", dir_name.as_str())))
                .context("failed to create new rumkinst directory")
                .fatal()?
        }
        Command::Make { path } => command_make(path)
            .context("failed to make installer artifacts with rumkinst")
            .fatal()?,
    }

    Ok(())
}

fn command_new(name: Identifier, dir_path: PathBuf) -> Result<()> {
    log::trace!("Running command logic for `new`");
    log::info!("Creating a new rumkinst directory...");

    if dir_path.exists() {
        anyhow::bail!("cannot create directory at {dir_path:?}, one already exists");
    }

    create_dir_with_context(dir_path.clone())?;

    create_dir_with_context(dir_path.join("root"))?;
    create_dir_with_context(dir_path.join("env"))?;
    create_dir_with_context(dir_path.join("scripts"))?;

    let config_file = File::create_new(dir_path.join("rumkinst.toml"))
        .with_context(|| format!("failed to create `rumkinst.toml` inside {dir_path:?}"))?;

    Config::write_default(config_file, name)
        .context("failed to write default config to `rumkinst.toml`")?;

    log::info!("Succesfully created new rumkinst directory at {dir_path:?}");
    Ok(())
}

fn create_dir_with_context(dir_path: PathBuf) -> Result<()> {
    fs::create_dir(&dir_path).with_context(|| format!("failed to create directory at {dir_path:?}"))
}

fn command_make(path: Option<PathBuf>) -> Result<()> {
    log::trace!("Running command logic for `make`");
    let config_path = find_config_file_at(path)
        .context("could not find `rumkinst.toml` config file")
        .fatal()?;

    let config_file =
        File::open(&config_path).with_context(|| format!("failed to open {config_path:?}"))?;

    let config = Config::read(config_file)
        .with_context(|| format!("could not read rumkinst config at {config_path:?}"))?;

    move_to_config_parent(&config_path)
        .context("could not move to the parent directory of rumkinst.toml")?;

    Ok(())
}
