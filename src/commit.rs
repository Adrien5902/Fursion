use rand;
use serde::{Deserialize, Serialize};

/// A commmit with a message and an id
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Commit {
    /// The message associated with the commit
    pub message: String,
    /// A random 3 bytes id can be displayed as 6 digits hex
    pub id: [u8; 3],
}

impl Commit {
    /// Makes a new commit object with a pseudo-random id
    pub(crate) fn new(message: &str) -> Self {
        Commit {
            message: message.to_owned(),
            id: rand::random(),
        }
    }

    /// Gives back an hex value which equals to the commit random id
    /// The hex value looks something like ```69FC64```
    pub fn hex_id(&self) -> String {
        format!("{:X?}", self.id)
            .replace(", ", "")
            .strip_prefix("[")
            .unwrap()
            .strip_suffix("]")
            .unwrap()
            .to_owned()
    }
}
