use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote(String);

pub const REMOTE_FILE_NAME: &str = "remotes";

impl Remote {
    /// * `url` - the url of the remote must end with .fursion
    pub fn new(url: &str) -> Self {
        Self(url.to_string())
    }

    /// Gives back the remote url
    pub fn get(&self) -> String {
        self.0.clone()
    }
}
