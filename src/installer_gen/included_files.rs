use std::path::PathBuf;

use anyhow::{Context, Result};
use hashbrown::HashSet;

use crate::progress_log::set_progress_message;

pub(crate) struct ExclusionFilter {
    filter: HashSet<PathBuf>,
}

impl From<&Vec<PathBuf>> for ExclusionFilter {
    fn from(value: &Vec<PathBuf>) -> Self {
        ExclusionFilter {
            filter: HashSet::from_iter(value.iter().cloned()),
        }
    }
}

pub(crate) struct IncludedFiles {
    pub(crate) files: Vec<PathBuf>,
}

pub(crate) struct PathExplorer {
    root: PathBuf,
    filter: ExclusionFilter,
}

impl PathExplorer {
    pub(crate) fn new(root: PathBuf, filter: ExclusionFilter) -> Self {
        log::debug!("created new PathExplorer for {root:?}");
        Self { root, filter }
    }

    pub(crate) fn search(self) -> Result<IncludedFiles> {
        log::trace!("searching with PathExplorer");
        if self.root.is_dir() {
            log::debug!("path is a directory, searching recursively");
            Ok(IncludedFiles {
                files: visit_dirs(self.root, &self.filter)?,
            })
        } else if self.root.is_file() {
            log::debug!("path is a single file, using single item buffer");
            Ok(IncludedFiles {
                files: vec![self.root],
            })
        } else if !self.root.exists() {
            anyhow::bail!("failed to search {:?}, file path does not exist", self.root)
        } else {
            anyhow::bail!(
                "failed to search {:?}, encountered an unexpected error",
                self.root
            )
        }
    }
}

fn visit_dirs(path: PathBuf, filter: &ExclusionFilter) -> Result<Vec<PathBuf>> {
    log::trace!("visiting directory recursively from root");
    let mut buf = Vec::new();
    recurse_into(path, filter, &mut buf).context("error while visiting dir")?;
    Ok(buf)
}

fn recurse_into(path: PathBuf, filter: &ExclusionFilter, buf: &mut Vec<PathBuf>) -> Result<()> {
    log::trace!("searching directory recursively");
    log::debug!("searching items in {path:?}");
    for entry in path
        .read_dir()
        .with_context(|| format!("failed to read directory {path:?}"))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry inside of directory {path:?}"))?;
        let path = entry.path();

        if filter.filter.contains(&path) {
            log::debug!("found path {path:?} which is excluded by the filter, continuing");
            continue;
        }

        set_progress_message(format!("Reading {path:?}"));

        if path.is_file() {
            log::debug!("file at {path:?}, appending to file buffer");
            buf.push(path);
        } else if path.is_dir() {
            log::debug!("directory at {path:?}, searching directory contents recursively");
            recurse_into(path, filter, buf)?;
        } else {
            anyhow::bail!("failed to find file or directory to read at {path:?}");
        }
    }

    Ok(())
}
