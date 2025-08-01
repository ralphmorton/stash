use std::str::FromStr;

use bincode::{Decode, Encode};

use super::db;

pub const ALPN: &[u8] = b"stash";

pub type SHA256 = String;

#[derive(Clone, Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Clone, Debug, Decode, Encode)]
pub enum Cmd {
    AddClient {
        node: String,
    },
    RemoveClient {
        node: String,
    },
    Tags,
    CreateBlob,
    DescribeBlob {
        name: String,
    },
    AppendBlob {
        name: String,
        data: Vec<u8>,
    },
    CommitBlob {
        name: String,
        file_name: String,
        tags: Vec<String>,
        replace: bool,
    },
    GcBlobs,
    List {
        tag: String,
        prefix: Option<String>,
    },
    Search {
        tag: String,
        term: String,
    },
    Describe {
        name: String,
    },
    Delete {
        name: String,
    },
    Download {
        hash: SHA256,
        start: u64,
        len: u64,
    },
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum Response<R> {
    Ok(R),
    Err(String),
}

impl Response<String> {
    pub fn ok() -> Self {
        Self::Ok("OK".to_string())
    }
}

impl<R> Response<R> {
    pub fn res(self) -> anyhow::Result<R> {
        match self {
            Self::Ok(r) => Ok(r),
            Self::Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub fn unwrap(self) -> R {
        match self {
            Self::Ok(r) => r,
            Self::Err(_) => panic!("`unwrap` called on Response::Err"),
        }
    }

    pub fn err(self) -> String {
        match self {
            Self::Ok(_) => panic!("`err` called on Response::Ok"),
            Self::Err(e) => e,
        }
    }
}

#[derive(Clone, Debug, Decode, PartialEq, Encode)]
pub struct Tag(String);

impl Tag {
    pub fn tag(&self) -> &str {
        &self.0
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.0
    }
}

impl FromStr for Tag {
    type Err = ();

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        for c in tag.chars() {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '-' {
                return Err(());
            }
        }

        Ok(Self(tag.to_string()))
    }
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct File {
    pub name: String,
    pub size: u64,
    pub hash: SHA256,
    pub created: i64,
}

impl From<db::FileDesc> for File {
    fn from(value: db::FileDesc) -> Self {
        Self {
            name: value.name,
            size: value.size as u64,
            hash: value.hash,
            created: value.created.and_utc().timestamp(),
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct FileDescription {
    pub name: String,
    pub size: u64,
    pub hash: SHA256,
    pub created: i64,
    pub tags: Vec<String>,
}

impl FileDescription {
    pub fn new(file: db::FileDesc, tags: Vec<String>) -> Self {
        Self {
            name: file.name,
            size: file.size as u64,
            hash: file.hash,
            created: file.created.and_utc().timestamp(),
            tags,
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct Blob {
    pub name: String,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use crate::Tag;
    use std::str::FromStr;

    #[test]
    fn tag_validation() {
        let t = Tag::from_str("test-1");
        assert!(matches!(t, Ok(_)));
        assert_eq!(t.unwrap().tag(), "test-1");

        let t = Tag::from_str("1-test");
        assert!(matches!(t, Ok(_)));
        assert_eq!(t.unwrap().tag(), "1-test");

        let t = Tag::from_str(";notvalid");
        assert!(matches!(t, Err(_)));
    }
}
