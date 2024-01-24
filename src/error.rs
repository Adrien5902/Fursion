use actix_web::{http::header::ContentType, HttpResponse};
use reqwest::StatusCode;
use serde::Serialize;
use std::{fmt::Display, path::PathBuf, str::Utf8Error};

#[derive(Debug, Serialize)]
pub enum Error {
    RepoReadFailed(RepoErrorReason),
    RepoInitFailed(RepoErrorReason),
    RepoFetchFailed(String),
    HostError(HostErrorKind),
    CommitParseFailed(CommitParseFailedReason),
    SerdeError(String),
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

macro_rules! impl_from_error {
    ($from_type:ty) => {
        impl From<$from_type> for Error {
            fn from(value: $from_type) -> Self {
                Error::Unknown(value.to_string())
            }
        }
    };
    ($from_type:ty, $error_type:ident) => {
        impl From<$from_type> for Error {
            fn from(value: $from_type) -> Self {
                Error::$error_type(value.to_string())
            }
        }
    };
}

impl_from_error!(Utf8Error);
impl_from_error!(std::io::Error);
impl_from_error!(serde_json::Error, SerdeError);

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

#[derive(Debug, Serialize)]
pub enum CommitParseFailedReason {
    CommitIdParseFailed,
    FileChangeDataNotFound,
    FileChangeDataMalformed,
}
