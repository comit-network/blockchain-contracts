#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    missing_copy_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::print_stdout,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

mod calculate_offsets;

use crate::calculate_offsets::placeholder_offsets;
use crate::calculate_offsets::{
    bitcoin::BitcoinScript, ethereum::EthereumContract, offset::to_markdown, Contract,
};
use std::ffi::OsStr;

const HETH_TEMPLATE_FOLDER: &str = "./print_offsets/heth_template/";
const HERC20_TEMPLATE_FOLDER: &str = "./print_offsets/herc20_template/";
const HBIT_TEMPLATE_FOLDER: &str = "./print_offsets/hbit_template/";

#[allow(clippy::print_stdout)]
fn main() -> Result<(), Error> {
    println!(
        "{}",
        generate_markdown::<BitcoinScript, &str>(HBIT_TEMPLATE_FOLDER)?
    );
    println!(
        "{}",
        generate_markdown::<EthereumContract, &str>(HETH_TEMPLATE_FOLDER)?
    );
    println!(
        "{}",
        generate_markdown::<EthereumContract, &str>(HERC20_TEMPLATE_FOLDER)?
    );

    Ok(())
}

fn generate_markdown<C: Contract, S: AsRef<OsStr>>(template_folder: S) -> Result<String, C::Error> {
    let contract = C::compile(template_folder)?;

    let metadata = contract.metadata();
    let offsets = placeholder_offsets(contract)?;

    Ok(format!(
        "{}\n{}",
        metadata.to_markdown(),
        to_markdown(offsets)
    ))
}

#[derive(Debug)]
enum Error {
    BitcoinScript(self::calculate_offsets::bitcoin::Error),
    EthereumContract(self::calculate_offsets::ethereum::Error),
}

impl From<self::calculate_offsets::bitcoin::Error> for Error {
    fn from(err: self::calculate_offsets::bitcoin::Error) -> Self {
        Error::BitcoinScript(err)
    }
}

impl From<self::calculate_offsets::ethereum::Error> for Error {
    fn from(err: self::calculate_offsets::ethereum::Error) -> Self {
        Error::EthereumContract(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_contracts::ethereum::{herc20, heth};

    #[test]
    fn heth_contract_template_matches_template_in_calculate_offsets() -> Result<(), Error> {
        let contract = EthereumContract::compile(HETH_TEMPLATE_FOLDER)?;
        assert_eq!(
            heth::CONTRACT_TEMPLATE.to_vec(),
            contract.metadata().contract,
        );
        Ok(())
    }

    #[test]
    fn herc20_contract_template_matches_template_in_calculate_offsets() -> Result<(), Error> {
        let contract = EthereumContract::compile(HERC20_TEMPLATE_FOLDER)?;
        assert_eq!(
            herc20::CONTRACT_TEMPLATE.to_vec(),
            contract.metadata().contract,
        );
        Ok(())
    }
}
