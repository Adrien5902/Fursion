use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    RepoReadFailed(RepoErrorReason),
    RepoInitFailed(RepoErrorReason),
    Unknown(String),
}

#[derive(Debug)]
pub enum RepoErrorReason {
    CantCreateFile(PathBuf, String),
    CantReadFile(PathBuf, String),
    PathNotFound(PathBuf),
    DirIsNotAFursionRepo(PathBuf),
    FailedToReadFileMetadata(String),
}
