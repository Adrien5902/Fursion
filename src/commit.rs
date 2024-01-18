#[derive(Debug)]
pub struct Commit {
    message: String,
    id: u32,
}

impl Commit {
    fn new(message: &str) -> Self {
        Commit {
            message: message.to_owned(),
            id: ,
        }
    }
}
