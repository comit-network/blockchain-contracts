use anyhow::{Context, Result};
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

impl PlaceholderConfig {
    pub fn from_file<S: AsRef<Path>>(file_path: S) -> Result<PlaceholderConfig> {
        let file_path = file_path.as_ref();
        let file = File::open(file_path).with_context(|| {
            format!(
                "failed to open placeholder config at {}",
                file_path.display()
            )
        })?;
        let reader = BufReader::new(file);

        let config =
            serde_json::from_reader(reader).context("failed to deserialize placeholder config")?;

        Ok(config)
    }
}
