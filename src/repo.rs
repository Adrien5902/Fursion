use std::{
    ffi::{OsStr, OsString},
    fs, mem,
    path::{Path, PathBuf},
};

use futures;
use serde::{Deserialize, Serialize};

use crate::{
    commit::{Commit, FileChange, FileChanges},
    error::{Error, RepoErrorReason},
    remote::Remote,
};

pub const FURSION_DIR: &str = ".fursion";

/// Repository Object
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Repo {
    /// List of files in the Repo
    pub files: Vec<File>,
    /// List of the remotes connected to this repo
    pub remotes: Vec<Remote>,
    /// The commit history of the repo
    pub history: RepoHistory,
    /// The repo metadata mainly author name and repo name
    pub metadata: RepoMetadata,
    /// List of stated changes in the repo
    stated_changes: FileChanges,
    /// The path leading to the repo on the system
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoMetadata {
    pub author: String,
    pub name: String,
}

impl RepoMetadata {
    pub const FILE_NAME: &'static str = "metadata";
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct File {
    pub path: PathBuf,
    pub name: OsString,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoHistory {
    vec: Vec<Commit>,
}

impl RepoHistory {
    pub const FILE_NAME: &'static str = "history";

    fn new() -> Self {
        RepoHistory { vec: Vec::new() }
    }

    fn push(&mut self, commit: Commit) {
        self.vec.push(commit)
    }

    pub fn read(path: &Path) -> Result<Self, Error> {
        let data = fs::read(path)?;
        let s = std::str::from_utf8(&data)?;
        Self::from_str(s)
    }

    pub fn to_string(&self) -> String {
        self.vec
            .iter()
            .map(|commit| commit.to_string())
            .collect::<Vec<_>>()
            .join(Commit::DELIMITER)
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let commits = s
            .split(Commit::DELIMITER)
            .map(|commit_str| Commit::from_str(commit_str))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { vec: commits })
    }
}

const EXCLUDE_FURSION_DIR: fn(&OsStr) -> bool = |file_name| file_name != FURSION_DIR;

impl Repo {
    const STATED_CHANGES_FILE_NAME: &'static str = "stated";

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

        let remotes_path = fursion_dir.join(Remote::FILE_NAME);
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

        let metadata_path = fursion_dir.join(RepoMetadata::FILE_NAME);
        let metadata = serde_json::from_slice(&fs::read(metadata_path.clone()).map_err(|e| {
            Error::RepoReadFailed(RepoErrorReason::CantReadFile(
                metadata_path.clone(),
                e.to_string(),
            ))
        })?)
        .map_err(|e| {
            Error::RepoReadFailed(RepoErrorReason::CantReadFile(
                metadata_path.clone(),
                e.to_string(),
            ))
        })?;

        let state_path = fursion_dir.join("state");
        let stated_changes = FileChanges::from_file(&state_path)?;

        let history = RepoHistory::read(&fursion_dir.join(RepoHistory::FILE_NAME))?;

        Ok(Repo {
            path: path.to_owned(),
            stated_changes,
            metadata,
            files,
            remotes,
            history,
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

        let repo = Repo {
            path: path.to_owned(),
            metadata: RepoMetadata {
                author: "".to_string(),
                name: path
                    .file_name()
                    .ok_or(Error::RepoInitFailed(
                        RepoErrorReason::CantInitAtRootDiskLocation,
                    ))?
                    .to_string_lossy()
                    .to_string(),
            },
            stated_changes: FileChanges::default(),
            remotes: Vec::new(),
            files: recursive_read_dir(path, EXCLUDE_FURSION_DIR)?,
            history: RepoHistory::new(),
        };

        repo.save()?;

        Ok(repo)
    }

    pub fn commit(&mut self, message: &str) -> Result<(), Error> {
        let changes = if !self.stated_changes.is_empty() {
            mem::replace(&mut self.stated_changes, FileChanges::default())
        } else {
            self.get_diff()?
        };

        let commit = Commit::new(message, changes);

        self.history.push(commit);
        Ok(())
    }

    ///WIP
    pub fn get_diff(&self) -> Result<FileChanges, Error> {
        Ok(FileChanges::default())
    }

    pub fn save(&self) -> Result<(), Error> {
        let fursion_dir = self.path.join(FURSION_DIR);

        fs::write(
            fursion_dir.join(RepoHistory::FILE_NAME),
            self.history.to_string(),
        )?;

        fs::write(fursion_dir.join(Remote::FILE_NAME), "")?; // WIP

        fs::write(
            fursion_dir.join(RepoMetadata::FILE_NAME),
            serde_json::to_string(&self.metadata)?,
        )?;

        fs::write(
            fursion_dir.join(Self::STATED_CHANGES_FILE_NAME),
            self.stated_changes.to_string(),
        )?;

        Ok(())
    }

    pub fn pull(&mut self) {}

    pub async fn fetch_remote(&self, remote: &Remote) -> Result<Repo, Error> {
        let res = reqwest::get(remote.get())
            .await
            .map_err(|e| Error::RepoFetchFailed(e.to_string()))?;

        let repo = res
            .json()
            .await
            .map_err(|e| Error::RepoFetchFailed(e.to_string()))?;

        Ok(repo)
    }

    pub async fn fetch(&self) -> Vec<Result<Repo, Error>> {
        let iter = self.remotes.iter().map(|r| self.fetch_remote(r));
        futures::future::join_all(iter).await
    }

    /// Reloads the repo's data by mutating, and returns the old data
    pub fn reread(&mut self) -> Result<Self, Error> {
        let new_read = Self::read(&self.path)?;
        Ok(mem::replace(self, new_read))
    }
}

/// Recursively reads a directory outputting a vec of {File} object
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
