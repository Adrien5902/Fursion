use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote(String);

pub const REMOTE_FILE_NAME: &str = "remotes";

impl Remote {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn get(&self) -> String {
        self.0.clone()
    }
}
