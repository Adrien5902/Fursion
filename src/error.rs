use actix_web::{http::header::ContentType, HttpResponse};
use reqwest::StatusCode;
use serde::Serialize;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Serialize)]
pub enum Error {
    RepoReadFailed(RepoErrorReason),
    RepoInitFailed(RepoErrorReason),
    RepoFetchFailed(String),
    HostError(HostErrorKind),
    Unknown(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(match self {
            Error::HostError(HostErrorKind::RepoNotFound) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
        .insert_header(ContentType::json())
        .body(serde_json::to_string(self).unwrap())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Unknown(value.to_string())
    }
}

#[derive(Debug, Serialize)]
pub enum RepoErrorReason {
    CantCreateFile(PathBuf, String),
    CantReadFile(PathBuf, String),
    PathNotFound(PathBuf),
    DirIsNotAFursionRepo(PathBuf),
    FailedToReadFileMetadata(String),
    CantInitAtRootDiskLocation,
}

#[derive(Debug, Serialize)]
pub enum HostErrorKind {
    RepoNotFound,
    StateMutexThreadLocked,
}
