use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use futures;
use serde::Deserialize;

use crate::{
    commit::Commit,
    error::{Error, RepoErrorReason},
    remote::{Remote, REMOTE_FILE_NAME},
};

pub const FURSION_DIR: &str = ".fursion";

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub files: Vec<File>,
    pub remotes: Vec<Remote>,
    pub history: Vec<Commit>,
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub path: PathBuf,
    pub name: OsString,
}

const EXCLUDE_FURSION_DIR: fn(&OsStr) -> bool = |file_name| file_name != FURSION_DIR;

impl Repo {
    pub fn read(path: &Path) -> Result<Self, Error> {
        if !Path::exists(path) {
            return Err(Error::RepoReadFailed(RepoErrorReason::PathNotFound(
                path.to_owned(),
            )));
        }

        let fursion_dir = path.join(FURSION_DIR);
        if !Path::exists(&fursion_dir) {
            return Err(Error::RepoReadFailed(
                RepoErrorReason::DirIsNotAFursionRepo(path.to_owned()),
            ));
        }

        let remotes_path = fursion_dir.join(REMOTE_FILE_NAME);
        let remotes_data = if Path::exists(&remotes_path) {
            fs::read(remotes_path.clone()).map_err(|e| {
                Error::RepoReadFailed(RepoErrorReason::CantReadFile(
                    remotes_path.clone(),
                    e.to_string(),
                ))
            })
        } else {
            Ok(Vec::new())
        }?;

        let remotes_str =
            std::str::from_utf8(&remotes_data).map_err(|e| Error::Unknown(e.to_string()))?;
        let remotes = remotes_str.lines().map(Remote::new).collect();

        let files = recursive_read_dir(path, EXCLUDE_FURSION_DIR)?;

        Ok(Repo {
            files,
            remotes,
            history: Vec::new(),
        })
    }

    pub fn init(path: &Path) -> Result<Self, Error> {
        let fursion_path = path.join(FURSION_DIR);

        fs::create_dir_all(&fursion_path).map_err(|e| {
            Error::RepoInitFailed(RepoErrorReason::CantCreateFile(
                path.to_owned(),
                e.to_string(),
            ))
        })?;

        Ok(Repo {
            remotes: Vec::new(),
            files: recursive_read_dir(path, EXCLUDE_FURSION_DIR)?,
            history: Vec::new(),
        })
    }

    pub fn commit(&mut self, message: &str) -> Result<(), Error> {
        let commit = Commit::new(message);

        self.history.push(commit);
        Ok(())
    }

    pub fn push(&self) {
        self.history.iter().for_each(|commit| {})
    }

    pub fn pull(&mut self) {}

    pub async fn fetch(&self) -> Vec<Result<Repo, Error>> {
        async fn fetch_remote(remote: &Remote) -> Result<Repo, Error> {
            let res = reqwest::get(remote.get())
                .await
                .map_err(|e| Error::RepoFetchFailed(e.to_string()))?;

            let repo = res
                .json()
                .await
                .map_err(|e| Error::RepoFetchFailed(e.to_string()))?;
            Ok(repo)
        }

        let iter = self.remotes.iter().map(fetch_remote);
        futures::future::join_all(iter).await
    }
}

fn recursive_read_dir(
    path: &Path,
    exclude: fn(entry_name: &OsStr) -> bool,
) -> Result<Vec<File>, Error> {
    let files = fs::read_dir(path)
        .map_err(|_| Error::RepoReadFailed(RepoErrorReason::PathNotFound(path.to_owned())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            Error::RepoReadFailed(RepoErrorReason::CantReadFile(
                path.to_owned(),
                e.to_string(),
            ))
        })?;

    files
        .iter()
        .filter(|f| exclude(&f.file_name()))
        .map(|entry| {
            let metadata = entry.metadata().map_err(|e| {
                Error::RepoReadFailed(RepoErrorReason::FailedToReadFileMetadata(e.to_string()))
            })?;

            if metadata.is_dir() {
                recursive_read_dir(&entry.path(), exclude)
            } else {
                Ok(vec![File {
                    path: entry.path(),
                    name: entry.file_name(),
                }])
            }
        })
        .try_fold(Vec::new(), |mut acc, result| {
            let vec = result?;
            acc.extend(vec);
            Ok(acc)
        })
}
