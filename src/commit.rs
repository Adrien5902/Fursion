use rand;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Commit {
    message: String,
    id: [u16; 5],
}

impl Commit {
    pub fn new(message: &str) -> Self {
        Commit {
            message: message.to_owned(),
            id: rand::random(),
        }
    }
}
