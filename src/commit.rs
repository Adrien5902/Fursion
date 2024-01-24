use rand;
use serde::{Deserialize, Serialize};
use std::{fs, ops::Range, panic::catch_unwind, path::Path};

use crate::error::{CommitParseFailedReason, Error};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CommitId([u8; 3]);

impl CommitId {
    /// Init function which outputs a random commit id
    fn new() -> Self {
        Self(rand::random())
    }

    /// Gives back an hex value which equals to the commit id
    /// The hex value looks something like ```69FC64```
    pub fn to_hex(&self) -> String {
        format!("{:X?}", self.0)
            .replace(", ", "")
            .strip_prefix("[")
            .unwrap()
            .strip_suffix("]")
            .unwrap()
            .to_owned()
    }

    pub fn from_hex(s: &str) -> Result<Self, Error> {
        let vec = vec![&s[0..=1], &s[2..=3], &s[4..=5]];

        let num_vec = vec
            .iter()
            .map(|s| u8::from_str_radix(s, 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::CommitParseFailed(CommitParseFailedReason::CommitIdParseFailed))?;

        let slice = num_vec
            .try_into()
            .map_err(|_| Error::CommitParseFailed(CommitParseFailedReason::CommitIdParseFailed))?;

        Ok(CommitId(slice))
    }
}

/// A commit with a message and an id
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Commit {
    /// The message associated with the commit
    pub message: String,
    /// A random 3 bytes id can be displayed as 6 digits hex
    pub id: CommitId,
    changes: FileChanges,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct FileChanges(Vec<FileChange>);
impl FileChanges {
    pub fn new(value: Vec<FileChange>) -> Self {
        Self(value)
    }

    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let file_data = fs::read(path)?;
        let file_str = std::str::from_utf8(&file_data)?;
        Self::from_str(file_str)
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let changes_iter = s.split(FileChange::DELIMITER);
        let vec = changes_iter
            .map(FileChange::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(vec))
    }

    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(FileChange::to_string)
            .collect::<Vec<_>>()
            .join(FileChange::DELIMITER)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileChange {
    pub file_path: String,
    pub range: Range<usize>,
    pub text: Option<String>,
    pub operation: FileChangeOperation,
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
    pub const DELIMITER: &'static str = "\nEND_FURSION_CHANGE";

    pub fn from_str(str: &str) -> Result<FileChange, Error> {
        catch_unwind(|| {
            let meta_str = str
                .lines()
                .collect::<Vec<_>>()
                .first()
                .and_then(|s| Some(*s))
                .ok_or(Error::CommitParseFailed(
                    CommitParseFailedReason::FileChangeDataNotFound,
                ))?;

            let meta = meta_str.split("|").collect::<Vec<_>>();

            let operation = FileChangeOperation::from_str(meta[2]);

            let text = if operation == FileChangeOperation::Deletion {
                None
            } else {
                Some(str[meta_str.len()..].to_string())
            };

            let range_nums: Vec<usize> = meta[1]
                .split("..")
                .map(str::parse::<usize>)
                .collect::<Result<_, _>>()
                .map_err(|_| {
                    Error::CommitParseFailed(CommitParseFailedReason::FileChangeDataMalformed)
                })?;

            let range = range_nums[0]..range_nums[1];

            Ok(FileChange {
                file_path: meta[0].to_string(),
                range,
                text,
                operation,
            })
        })
        .map_err(|_| Error::CommitParseFailed(CommitParseFailedReason::FileChangeDataMalformed))?
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
    pub const DELIMITER: &'static str = "\nFURSION_COMMIT";

    /// Makes a new commit object with a pseudo-random id
    pub(crate) fn new(message: &str, changes: FileChanges) -> Self {
        Commit {
            changes,
            message: message.to_owned(),
            id: CommitId::new(),
        }
    }

    pub fn to_string(&self) -> String {
        let changes_str = self
            .changes
            .0
            .iter()
            .map(|change| change.to_string())
            .collect::<Vec<_>>()
            .join(FileChange::DELIMITER);

        format!("{}", changes_str)
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let data_str = s.lines().collect::<Vec<_>>()[0];
        let data = data_str.split("|").collect::<Vec<_>>();

        let changes = s[data_str.len()..]
            .split(FileChange::DELIMITER)
            .map(FileChange::from_str)
            .collect::<Result<_, _>>()?;

        let message = data[1].to_string();
        let id = CommitId::from_hex(data[0])?;

        Ok(Self {
            message,
            id,
            changes: FileChanges::new(changes),
        })
    }
}
