mod cli;

use std::{
    fs::{self, File},
    io::{Seek, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use cli::{Command, Rumkinst};
use flate2::{Compression, GzBuilder};
use nanoid::nanoid;
use rumkinst::{
    config::{Config, find_config_file_at, identifier::Identifier},
    error_log::Log,
    installer_gen::{RumkinstFiles, find_all_files},
    progress_log::{progress_wrapper, setup_log_wrapper},
};
use sha2::{Digest, Sha256};

fn setup_logging(config: &Rumkinst) {
    let logger = env_logger::Builder::from_env(
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
    .build();
    let filter = logger.filter();

    setup_log_wrapper(logger, filter);
}

fn move_to_config_parent(path: &Path) -> Result<()> {
    log::trace!("moving working directory");
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
    log::trace!("running command logic for `new`");
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
    log::trace!("running command logic for `make`");
    let config_path =
        find_config_file_at(path).context("could not find `rumkinst.toml` config file")?;

    let config_file =
        File::open(&config_path).with_context(|| format!("failed to open {config_path:?}"))?;

    let config = Config::read(config_file)
        .with_context(|| format!("could not read rumkinst config at {config_path:?}"))?;

    move_to_config_parent(&config_path)
        .context("could not move to the parent directory of rumkinst.toml")?;

    let run_id = nanoid!();
    let out_dir = PathBuf::from(format!("./out/{run_id}"));
    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create output directory {out_dir:?}"))?;

    log::info!("Reading source directories");
    let all_files = progress_wrapper(3, || find_all_files(&config))
        .context("could not find all files for packaging")?;

    log::info!("Making rumkinst artifacts...");

    if all_files.total_files() > 0 {
        progress_wrapper(all_files.total_files() as u64, || {
            make_archive(&config, &out_dir, &all_files)
        })
        .context("failed to make archive file")?;
    } else {
        log::warn!("no source files included, skipping making archive file");
    }

    log::info!("Finished: artifacts available in output directory \"{run_id}\"");

    Ok(())
}

fn make_archive(config: &Config, out_dir: &Path, all_files: &RumkinstFiles) -> Result<()> {
    let archive_name = format!("{}.tar.gz", config.get_name());
    let checksum_name = format!("{archive_name}.sha256");

    let archive_path = out_dir.join(&archive_name);
    let checksum_path = out_dir.join(&checksum_name);

    log::info!("Making archive \"{archive_name}\"");

    let archive_file = File::create_new(&archive_path)
        .with_context(|| format!("failed to create new archive file at {archive_path:?}"))?;
    let mut checksum_file = File::create_new(&checksum_path)
        .with_context(|| format!("failed to create new checksum file at {checksum_path:?}"))?;
    let mut encoder = GzBuilder::new()
        .filename(archive_name.as_str())
        .write(archive_file, Compression::best());
    all_files
        .write_archive(&mut encoder)
        .with_context(|| format!("failed to write archive to {archive_path:?}"))?;
    let mut finished_file = encoder
        .finish()
        .context("failed to finish gzip encoding of archive")?;

    finished_file
        .seek(std::io::SeekFrom::Start(0))
        .context("failed to seek archive to start for checksum generation")?;

    let mut sha256 = Sha256::new();
    std::io::copy(&mut finished_file, &mut sha256)
        .context("failed to copy archive file into hasher")?;
    let digest = sha256.finalize();

    checksum_file
        .write_fmt(format_args!("{digest:x}  {archive_name}"))
        .with_context(|| format!("failed to write checksum to {checksum_path:?}"))?;

    Ok(())
}
