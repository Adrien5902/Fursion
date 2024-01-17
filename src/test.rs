use std::path::Path;

use crate::repo::Repo;

#[test]
fn main() {
    let repo = Repo::read(Path::new("C:\\Users\\adrie\\Desktop\\Some Folder")).unwrap();
    panic!("{:?}", repo.remotes);
}
