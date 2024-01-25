use std::{fs, path::Path};

use crate::{
    commit::{Commit, CommitId, FileChanges},
    repo::{self, Repo},
    server::Server,
};

#[test]
fn read() {
    let repo = Repo::read(Path::new("C:\\Users\\adrie\\Desktop\\Some Folder")).unwrap();
    panic!("{:?}", repo.remotes);
}

#[test]
fn main() {
    let path = Path::new("C:\\Users\\adrie\\Desktop\\Some Folder");
    fs::remove_dir_all(path.join(repo::FURSION_DIR)).unwrap();
    let repo = Repo::init(&path).unwrap();
    println!("{:?}", repo);
}

#[test]
/// Tests if commit id to/from hex conversion works
fn commit_id_hex() {
    let commit = Commit::new("test", FileChanges::default());
    assert_eq!(commit.id, CommitId::from_hex(&commit.id.to_hex()).unwrap());
}

#[actix_web::test]
async fn host() {
    Server::new().await.unwrap();
}
