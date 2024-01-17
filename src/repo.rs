use std::{
    ffi::{OsStr, OsString},
    fs::{self, Metadata},
    path::{Path, PathBuf},
};

pub const FURSION_DIR: &'static str = ".fursion";

use crate::{
    error::{Error, RepoReadFailedReason},
    remote::{Remote, REMOTE_FILE_NAME},
};

#[derive(Debug)]
pub struct Repo {
    pub files: Vec<File>,
    pub remotes: Vec<Remote>,
}

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
    pub name: OsString,
    pub metadata: Metadata,
}

impl Repo {
    pub fn read(path: &Path) -> Result<Self, Error> {
        if !Path::exists(path) {
            return Err(Error::RepoReadFailed(RepoReadFailedReason::PathNotFound(
                path.to_owned(),
            )));
        }

        let fursion_dir = path.join(FURSION_DIR);
        if !Path::exists(&fursion_dir) {
            return Err(Error::RepoReadFailed(
                RepoReadFailedReason::DirIsNotAFursionRepo(path.to_owned()),
            ));
        }

        let remotes_path = fursion_dir.join(REMOTE_FILE_NAME);
        let remotes_data = if Path::exists(&remotes_path) {
            fs::read(remotes_path.clone()).map_err(|e| {
                Error::RepoReadFailed(RepoReadFailedReason::CantReadFile(
                    remotes_path.clone(),
                    e.to_string(),
                ))
            })
        } else {
            Ok(Vec::new())
        }?;

        let remotes_str =
            std::str::from_utf8(&remotes_data).map_err(|e| Error::Unknown(e.to_string()))?;
        let remotes = remotes_str.lines().map(|s| Remote::new(s)).collect();

        let files = recursive_read_dir(path, |file_name| file_name != FURSION_DIR)?;

        Ok(Repo { files, remotes })
    }
}

fn recursive_read_dir(
    path: &Path,
    exclude: fn(entry_name: &OsStr) -> bool,
) -> Result<Vec<File>, Error> {
    let files = fs::read_dir(path)
        .map_err(|_| Error::RepoReadFailed(RepoReadFailedReason::PathNotFound(path.to_owned())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            Error::RepoReadFailed(RepoReadFailedReason::CantReadFile(
                path.to_owned(),
                e.to_string(),
            ))
        })?;

    files
        .iter()
        .filter(|f| exclude(&f.file_name()))
        .map(|entry| {
            let metadata = entry.metadata().map_err(|e| {
                Error::RepoReadFailed(RepoReadFailedReason::FailedToReadFileMetadata(
                    e.to_string(),
                ))
            })?;

            if metadata.is_dir() {
                recursive_read_dir(&entry.path(), exclude)
            } else {
                Ok(vec![File {
                    path: entry.path(),
                    name: entry.file_name(),
                    metadata,
                }])
            }
        })
        .try_fold(Vec::new(), |mut acc, result| {
            let vec = result?;
            acc.extend(vec);
            Ok(acc)
        })
}
