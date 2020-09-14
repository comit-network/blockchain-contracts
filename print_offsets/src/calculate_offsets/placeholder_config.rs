use serde::Deserialize;
use std::path::Path;
use std::{fs::File, io::BufReader};

#[derive(Debug, Deserialize)]
pub struct PlaceholderConfig {
    pub protocol_name: String,
    pub placeholders: Vec<Placeholder>,
}

#[derive(Debug, Deserialize)]
pub struct Placeholder {
    pub name: String,
    pub replace_pattern: String,
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    MalformedConfig(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::MalformedConfig(e)
    }
}

impl PlaceholderConfig {
    pub fn from_file<S: AsRef<Path>>(file_path: S) -> Result<PlaceholderConfig, Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let config = serde_json::from_reader(reader)?;

        Ok(config)
    }
}
