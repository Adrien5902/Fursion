use std::path::Path;

use crate::repo::Repo;

#[test]
fn read() {
    let repo = Repo::read(Path::new("C:\\Users\\adrie\\Desktop\\Some Folder")).unwrap();
    panic!("{:?}", repo.remotes);
}

#[test]
fn init() {
    let repo = Repo::init(Path::new("C:\\Users\\Administrateur\\Desktop\\Test")).unwrap();
}
