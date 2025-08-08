pub mod identifier;
mod relativepathbuf;

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use identifier::Identifier;
use log::{debug, trace};
use relativepathbuf::RelativePathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct InternalPackageDetails {
    name: Identifier,
    description: Option<String>,
    authors: Option<Vec<String>>,
}

#[derive(Debug)]
pub(crate) struct PackageDetails {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) authors: Vec<String>,
}

impl PackageDetails {
    fn init(source: InternalPackageDetails) -> Self {
        Self {
            name: source.name.into_string(),
            description: source.description,
            authors: source.authors.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) enum ThemeType {
    #[default]
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "box")]
    Box,
    #[serde(rename = "figlet")]
    Figlet,
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalInstallerConfig {
    #[serde(rename = "allow-user-install")]
    allow_user_install: Option<bool>,
    theme: Option<ThemeType>,

    preinstall: Option<RelativePathBuf>,
    postinstall: Option<RelativePathBuf>,
}

#[derive(Debug, Default)]
pub(crate) struct InstallerConfig {
    pub(crate) allow_user_install: bool,
    pub(crate) theme: ThemeType,

    pub(crate) preinstall: Option<PathBuf>,
    pub(crate) postinstall: Option<PathBuf>,
}

impl InstallerConfig {
    fn init(source: Option<InternalInstallerConfig>) -> Self {
        source
            .map(|source| Self {
                allow_user_install: source.allow_user_install.unwrap_or_default(),
                theme: source.theme.unwrap_or_default(),
                preinstall: source.preinstall.map(RelativePathBuf::into_pathbuf),
                postinstall: source.postinstall.map(RelativePathBuf::into_pathbuf),
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalBuildConfig {
    prebuild: Option<RelativePathBuf>,
    postbuild: Option<RelativePathBuf>,
}

#[derive(Debug)]
pub(crate) struct BuildConfig {
    pub(crate) prebuild: PathBuf,
    pub(crate) postbuild: PathBuf,
}

impl BuildConfig {
    fn init(source: Option<InternalBuildConfig>) -> Self {
        source
            .map(|source| Self {
                prebuild: source
                    .prebuild
                    .map(RelativePathBuf::into_pathbuf)
                    .unwrap_or(PathBuf::from("./prebuild.sh")),
                postbuild: source
                    .postbuild
                    .map(RelativePathBuf::into_pathbuf)
                    .unwrap_or(PathBuf::from("./postbuild.sh")),
            })
            .unwrap_or(Self {
                prebuild: PathBuf::from("./prebuild.sh"),
                postbuild: PathBuf::from("./postbuild.sh"),
            })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalSourceConfig {
    disable: Option<bool>,
    path: Option<RelativePathBuf>,
    exclude: Option<Vec<RelativePathBuf>>,
}

#[derive(Debug)]
pub(crate) struct SourceConfig {
    pub(crate) disable: bool,
    pub(crate) path: PathBuf,
    pub(crate) exclude: Vec<PathBuf>,
}

impl SourceConfig {
    fn init(source: Option<InternalSourceConfig>, default_path: &str) -> Self {
        match source {
            Some(source) => Self {
                disable: source.disable.unwrap_or(false),
                path: source
                    .path
                    .map(|rel| rel.into_pathbuf())
                    .unwrap_or(PathBuf::from(default_path)),
                exclude: source
                    .exclude
                    .map(|exclude| exclude.into_iter().map(|rel| rel.into_pathbuf()).collect())
                    .unwrap_or(vec![]),
            },
            None => Self {
                disable: false,
                path: PathBuf::from(default_path),
                exclude: vec![],
            },
        }
    }
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn exclude(&self) -> &Vec<PathBuf> {
        &self.exclude
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalConfig {
    package: InternalPackageDetails,

    installer: Option<InternalInstallerConfig>,
    build: Option<InternalBuildConfig>,

    root: Option<InternalSourceConfig>,
    env: Option<InternalSourceConfig>,
    scripts: Option<InternalSourceConfig>,
}

pub struct Config {
    pub(crate) package: PackageDetails,

    pub(crate) installer: InstallerConfig,
    pub(crate) build: BuildConfig,

    pub(crate) root: SourceConfig,
    pub(crate) env: SourceConfig,
    pub(crate) scripts: SourceConfig,
}

impl From<InternalConfig> for Config {
    fn from(value: InternalConfig) -> Self {
        Self {
            package: PackageDetails::init(value.package),

            installer: InstallerConfig::init(value.installer),
            build: BuildConfig::init(value.build),

            root: SourceConfig::init(value.root, "./root/"),
            env: SourceConfig::init(value.env, "./env/"),
            scripts: SourceConfig::init(value.scripts, "./scripts/"),
        }
    }
}

impl Config {
    pub fn read<R: Read>(mut readable: R) -> Result<Self> {
        trace!("reading config reader to config type");
        let mut config_str = String::new();
        readable
            .read_to_string(&mut config_str)
            .context("failed to finish reading reader to string")?;

        toml::from_str::<InternalConfig>(&config_str)
            .context("failed to parse rumkinst config from file text")
            .map(|cfg| {
                debug!("successfully parsed config");
                cfg.into()
            })
    }

    pub fn write_default<W: Write>(mut writable: W, package_name: Identifier) -> Result<()> {
        let config_str = toml::to_string_pretty(&InternalConfig {
            package: InternalPackageDetails {
                name: package_name,
                description: Some(String::new()),
                authors: Some(vec![]),
            },
            installer: Some(InternalInstallerConfig {
                allow_user_install: Some(false),
                theme: Some(ThemeType::Plain),

                preinstall: None,
                postinstall: None,
            }),
            build: None,
            root: None,
            env: None,
            scripts: None,
        })
        .context("failed to convert default config to toml string")?;

        writable
            .write_fmt(format_args!("{config_str}"))
            .context("failed to write default toml to writer")
    }

    pub fn get_name(&self) -> &str {
        &self.package.name
    }
}

pub fn find_config_file_at(path: Option<PathBuf>) -> Result<PathBuf> {
    trace!("searching for config file");
    debug!("provided path to search is `{path:?}`");

    match path {
        Some(path) => {
            if path.is_file() {
                debug!("provided path was a file");
                Ok(path)
            } else if path.is_dir() && path.join("rumkinst.toml").is_file() {
                debug!("provided path was a folder with `rumkinst.toml`");
                Ok(path.join("rumkinst.toml"))
            } else {
                debug!("provided path was not, or did not contain `rumkinst.toml`");
                anyhow::bail!("could not find `rumkinst.toml` in `{path:?}`");
            }
        }
        None => find_default_config_file()
            .context("could not find `rumkinst.toml` in `./` or in `./rumkinst/`"),
    }
}

fn find_default_config_file() -> Option<PathBuf> {
    trace!("searching default file paths for `rumkinst.toml`");
    config_exists("./").or_else(|| config_exists("./rumkinst/"))
}

fn config_exists(path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(path).join("rumkinst.toml");

    path.is_file().then_some(path)
}
