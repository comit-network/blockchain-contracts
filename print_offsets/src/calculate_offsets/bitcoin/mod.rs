use crate::calculate_offsets::{
    metadata::Metadata, placeholder_config::PlaceholderConfig, Contract,
};
use anyhow::Result;
use std::path::Path;

mod compile_contract;

pub struct BitcoinScript {
    bytes: Vec<u8>,
    placeholder_config: PlaceholderConfig,
}

impl Contract for BitcoinScript {
    fn compile<S: AsRef<Path>>(template_folder: S) -> Result<Self> {
        let bytes = compile_contract::compile(template_folder.as_ref().join("contract.script"))?;
        let placeholder_config =
            PlaceholderConfig::from_file(template_folder.as_ref().join("config.json"))?;

        Ok(Self {
            bytes,
            placeholder_config,
        })
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            protocol_name: self.placeholder_config.protocol_name.clone(),
            contract: self.bytes.to_owned(),
        }
    }

    fn placeholder_config(&self) -> &PlaceholderConfig {
        &self.placeholder_config
    }

    fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}
