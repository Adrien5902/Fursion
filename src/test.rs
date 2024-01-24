use std::path::Path;

use crate::{
    commit::{Commit, CommitId, FileChanges},
    repo::Repo,
    server::Server,
};

#[test]
fn read() {
    let repo = Repo::read(Path::new("C:\\Users\\adrie\\Desktop\\Some Folder")).unwrap();
    panic!("{:?}", repo.remotes);
}

#[test]
fn init() {
    let repo = Repo::init(Path::new("C:\\Users\\Administrateur\\Desktop\\Test")).unwrap();
}

#[test]
/// Tests if commit id to/from hex conversion works
fn commit_id_hex() {
    let commit = Commit::new("test", FileChanges::default());
    assert_eq!(commit.id, CommitId::from_hex(&commit.id.to_hex()).unwrap());
}

#[actix_web::test]
async fn host() {
    Server::new().await;
}
