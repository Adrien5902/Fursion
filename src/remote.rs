#[derive(Debug)]
pub struct Remote(String);

pub const REMOTE_FILE_NAME: &'static str = "remotes";

impl Remote {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}
