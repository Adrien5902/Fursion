use std::sync::Arc;

use actix_web::{get, web, App, HttpServer, Responder};
use futures::lock::Mutex;
use once_cell::sync::Lazy;

use crate::{
    error::{Error, HostErrorKind},
    repo::Repo,
};

static SERVER_STATE: Lazy<Arc<Mutex<ServerState>>> =
    Lazy::new(|| Arc::new(Mutex::new(ServerState::default())));

pub struct Server;

#[derive(Debug, Default)]
struct ServerState {
    pub repos: Vec<Repo>,
}

#[get("/{author}/{repo}")]
async fn get_repo(
    author: web::Path<String>,
    repo: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let server_state = SERVER_STATE
        .try_lock()
        .ok_or(Error::HostError(HostErrorKind::StateMutexThreadLocked))?;

    let repo_obj = server_state
        .repos
        .iter()
        .find(|r| r.metadata.name == *repo && r.metadata.author == *author)
        .ok_or(Error::HostError(HostErrorKind::RepoNotFound))
        .and_then(|r| Ok((*r).clone()))?;

    Ok(web::Json(repo_obj))
}

impl Server {
    pub async fn new() -> Result<Self, Error> {
        HttpServer::new(|| App::new().service(get_repo))
            .bind(("127.0.0.1", 54510))?
            .run()
            .await?;

        Ok(Server)
    }

    async fn reload(&mut self, repos: Vec<Repo>) -> Result<(), Error> {
        Ok(())
    }
}
