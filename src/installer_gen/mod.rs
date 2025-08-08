mod included_files;

use std::io::Write;

use anyhow::{Context, Result};
use included_files::{ExclusionFilter, IncludedFiles, PathExplorer};

use crate::{
    config::{Config, SourceConfig},
    progress_log::{increment_progress, set_progress_message},
};

pub struct RumkinstFiles {
    root_files: Option<IncludedFiles>,
    env_files: Option<IncludedFiles>,
    script_files: Option<IncludedFiles>,
}

impl RumkinstFiles {
    fn new(
        root_files: Option<IncludedFiles>,
        env_files: Option<IncludedFiles>,
        script_files: Option<IncludedFiles>,
    ) -> Self {
        Self {
            root_files,
            env_files,
            script_files,
        }
    }

    pub fn total_files(&self) -> usize {
        get_files_len(&self.root_files)
            + get_files_len(&self.env_files)
            + get_files_len(&self.script_files)
    }

    pub fn write_archive<W: Write>(&self, destination: W) -> Result<()> {
        let mut archive = tar::Builder::new(destination);

        write_archive(&self.root_files, &mut archive)?;
        write_archive(&self.env_files, &mut archive)?;
        write_archive(&self.script_files, &mut archive)?;

        archive.finish().context("failed to finish archive")?;

        Ok(())
    }
}

#[inline(always)]
fn get_files_len(opt: &Option<IncludedFiles>) -> usize {
    opt.as_ref().map(|files| files.files.len()).unwrap_or(0)
}

fn write_archive<W: Write>(
    opt: &Option<IncludedFiles>,
    archive: &mut tar::Builder<W>,
) -> Result<()> {
    if let Some(files) = opt {
        for path in files.files.iter() {
            set_progress_message(format!("Writing {path:?} to archive"));
            archive
                .append_path(path)
                .with_context(|| format!("failed to append {path:?} to archive"))?;
            increment_progress(1);
        }
    }

    Ok(())
}

pub fn find_all_files(config: &Config) -> Result<RumkinstFiles> {
    log::trace!("finding files for packaging");
    let root = search_source(&config.root).inspect(|_| increment_progress(1))?;
    let env = search_source(&config.env).inspect(|_| increment_progress(1))?;
    let script = search_source(&config.scripts).inspect(|_| increment_progress(1))?;

    Ok(RumkinstFiles::new(root, env, script))
}

fn search_source(source: &SourceConfig) -> Result<Option<IncludedFiles>> {
    log::trace!("searching a source");

    if source.disable {
        log::debug!("source.disable = true, skipping this source");
        return Ok(None);
    }

    let filter = ExclusionFilter::from(source.exclude());
    let explorer = PathExplorer::new(source.path().to_path_buf(), filter);
    explorer.search().map(Some)
}
