use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    hash: String,
    message: String,
}

impl Commit {
    pub fn new<S1: Into<String>, S2: Into<String>>(hash: S1, message: S2) -> Self {
        Self {
            hash: hash.into(),
            message: message.into(),
        }
    }
    pub fn hash(&self) -> &String {
        &self.hash
    }
    pub fn message(&self) -> &String {
        &self.message
    }
}
