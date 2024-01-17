use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    RepoReadFailed(RepoReadFailedReason),
    Unknown(String),
}

#[derive(Debug)]
pub enum RepoReadFailedReason {
    CantReadFile(PathBuf, String),
    PathNotFound(PathBuf),
    DirIsNotAFursionRepo(PathBuf),
    FailedToReadFileMetadata(String),
}
