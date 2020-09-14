use crate::calculate_offsets::{
    metadata::Metadata,
    offset::Offset,
    placeholder_config::{Placeholder, PlaceholderConfig},
};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub mod bitcoin;
pub mod ethereum;
pub mod metadata;
pub mod offset;
pub mod placeholder_config;

pub trait Contract: std::marker::Sized {
    fn compile<S: AsRef<Path>>(template_folder: S) -> Result<Self>;
    fn metadata(&self) -> Metadata;
    fn placeholder_config(&self) -> &PlaceholderConfig;
    fn bytes(&self) -> &[u8];
}

pub fn placeholder_offsets<C: Contract>(contract: C) -> Result<Vec<Offset>> {
    contract
        .placeholder_config()
        .placeholders
        .iter()
        .map(|placeholder| calc_offset(placeholder, contract.bytes()))
        .collect()
}

fn calc_offset(placeholder: &Placeholder, contract: &[u8]) -> Result<Offset> {
    let decoded_placeholder = hex::decode(placeholder.replace_pattern.as_str())?;
    let start_pos = find_subsequence(&contract[..], &decoded_placeholder[..])
        .with_context(|| format!("failed to find placeholder {}", placeholder.name))?;
    let end_pos = start_pos + decoded_placeholder.len();

    Ok(Offset {
        name: placeholder.name.to_owned(),
        start: start_pos,
        excluded_end: end_pos,
        length: decoded_placeholder.len(),
    })
}

fn find_subsequence(contract_template: &[u8], placeholder: &[u8]) -> Option<usize> {
    contract_template
        .windows(placeholder.len())
        .position(|window| window == placeholder)
}

fn check_bin_in_path(bin: &str) {
    let output = Command::new("which").arg(bin).output().unwrap();
    if output.stdout.is_empty() {
        let msg = format!(
            "`{}` cannot be found, check your path\nPATH: {:?}",
            bin,
            std::env::var("PATH")
        );
        panic!(msg);
    }
}
