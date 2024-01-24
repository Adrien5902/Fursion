use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote(String);

impl Remote {
    pub const FILE_NAME: &'static str = "remotes";

    /// * `url` - the url of the remote must end with .fursion
    pub fn new(url: &str) -> Self {
        Self(url.to_string())
    }

    /// Gives back the remote url
    pub fn get(&self) -> String {
        self.0.clone()
    }
}
