use rand;
use serde::{Deserialize, Serialize};
use std::{fs, ops::Range, panic::catch_unwind, path::Path};

use crate::error::{CommitParseFailedReason, Error};

/// A commit with a message and an id
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Commit {
    /// The message associated with the commit
    pub message: String,
    /// A random 3 bytes id can be displayed as 6 digits hex
    pub id: [u8; 3],
    changes: Vec<FileChange>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileChange {
    file_path: String,
    range: Range<usize>,
    text: Option<String>,
    operation: FileChangeOperation,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum FileChangeOperation {
    Insertion,
    Deletion,
}

impl FileChangeOperation {
    fn to_string(&self) -> String {
        match self {
            Self::Deletion => "DEL",
            Self::Insertion => "INS",
        }
        .to_string()
    }

    fn from_str(s: &str) -> Self {
        match s {
            "DEL" => Self::Deletion,
            "INS" => Self::Insertion,
            _ => panic!("Invalid FileChangeOperation string: {}", s),
        }
    }
}

impl From<&str> for FileChangeOperation {
    fn from(s: &str) -> Self {
        FileChangeOperation::from_str(s)
    }
}

impl Into<String> for FileChangeOperation {
    fn into(self) -> String {
        self.to_string()
    }
}

impl FileChange {
    pub const DELIMITER: &'static str = "END_FURSION_CHANGE";

    pub fn from_file(path: &Path) -> Result<Vec<Self>, Error> {
        let delimiter1 = "\n".to_string() + Self::DELIMITER;
        let delimiter2 = "\r\n".to_string() + Self::DELIMITER;

        let file_data = fs::read(path)?;
        let file_str = std::str::from_utf8(&file_data)?;
        let changes_iter = file_str
            .split(delimiter1.as_str())
            .map(|s| s.split(delimiter2.as_str()))
            .flatten();

        changes_iter.map(Self::from_str).collect()
    }

    pub fn from_str(str: &str) -> Result<FileChange, Error> {
        let mut lines = str.lines();

        let meta = lines
            .next()
            .ok_or(Error::CommitParseFailed(
                CommitParseFailedReason::FileChangeDataNotFound,
            ))?
            .split("|")
            .collect::<Vec<_>>();

        let operation = FileChangeOperation::from_str(meta[2]);

        let text = if operation == FileChangeOperation::Deletion {
            None
        } else {
            Some(lines.collect::<Vec<_>>().join("\n"))
        };

        let range_nums: Vec<usize> = meta[1]
            .split("..")
            .map(str::parse::<usize>)
            .collect::<Result<_, _>>()
            .map_err(|_| {
                Error::CommitParseFailed(CommitParseFailedReason::FileChangeDataMalformed)
            })?;

        let range = catch_unwind(|| range_nums[0]..range_nums[1]).map_err(|_| {
            Error::CommitParseFailed(CommitParseFailedReason::FileChangeDataMalformed)
        })?;

        Ok(FileChange {
            file_path: meta[0].to_string(),
            range,
            text,
            operation,
        })
    }

    pub fn to_string(&self) -> String {
        format!(
            "{}|{}..{}|{}",
            self.file_path,
            self.range.start,
            self.range.end,
            self.operation.to_string()
        )
    }
}

impl Commit {
    /// Makes a new commit object with a pseudo-random id
    pub(crate) fn new(message: &str, changes: Vec<FileChange>) -> Self {
        Commit {
            changes,
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

    pub fn to_string() {}
}
