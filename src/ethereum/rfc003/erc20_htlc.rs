use crate::ethereum::{Address, TokenQuantity};
use crate::{EthereumTimestamp, FitIntoPlaceholderSlice, SecretHash};
use hex_literal::hex;

// contract template RFC: https://github.com/comit-network/RFCs/blob/master/RFC-009-SWAP-Basic-ERC20.md#contract
pub const CONTRACT_TEMPLATE: [u8;411] = hex!("61018c61000f60003961018c6000f3361561007957602036141561004f57602060006000376020602160206000600060026048f17f100000000000000000000000000000000000000000000000000000000000000160215114166100ae575b7f696e76616c69645365637265740000000000000000000000000000000000000060005260206000fd5b426320000002106100f1577f746f6f4561726c7900000000000000000000000000000000000000000000000060005260206000fd5b7f72656465656d656400000000000000000000000000000000000000000000000060206000a1733000000000000000000000000000000000000003602052610134565b7f726566756e64656400000000000000000000000000000000000000000000000060006000a1734000000000000000000000000000000000000004602052610134565b63a9059cbb6000527f5000000000000000000000000000000000000000000000000000000000000005604052602060606044601c6000736000000000000000000000000000000000000006620186a05a03f150602051ff");

#[derive(Debug, Clone)]
pub struct Erc20Htlc(Vec<u8>);

impl From<Erc20Htlc> for Vec<u8> {
    fn from(htlc: Erc20Htlc) -> Self {
        htlc.0
    }
}

impl Erc20Htlc {
    pub fn new(
        expiry: u32,
        refund_identity: Address,
        redeem_identity: Address,
        secret_hash: [u8; 32],
        token_contract_address: Address,
        token_quantity: TokenQuantity,
    ) -> Self {
        let mut contract = CONTRACT_TEMPLATE.to_vec();
        SecretHash(secret_hash).fit_into_placeholder_slice(&mut contract[53..85]);
        EthereumTimestamp(expiry).fit_into_placeholder_slice(&mut contract[139..143]);
        redeem_identity.fit_into_placeholder_slice(&mut contract[229..249]);
        refund_identity.fit_into_placeholder_slice(&mut contract[296..316]);
        token_quantity.fit_into_placeholder_slice(&mut contract[333..365]);
        token_contract_address.fit_into_placeholder_slice(&mut contract[379..399]);

        Erc20Htlc(contract)
    }

    pub fn deployment_gas_limit(&self) -> u64 {
        167_800
    }

    pub fn tx_gas_limit() -> u64 {
        100_000
    }

    pub fn fund_tx_gas_limit() -> u64 {
        100_000
    }

    /// Constructs the payload to transfer `Erc20` tokens to a `to_address`
    /// Note: `token_quantity` must be BigEndian
    pub fn transfer_erc20_tx_payload(
        token_quantity: TokenQuantity,
        to_address: Address,
    ) -> Vec<u8> {
        let transfer_fn_abi = hex!("A9059CBB");

        let mut data = [0u8; 4 + 32 + 32];
        data[..4].copy_from_slice(&transfer_fn_abi);
        data[16..36].copy_from_slice(&to_address.0);
        data[36..68].copy_from_slice(&token_quantity.0);

        data.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::{FromHex, ToHex};
    use regex::bytes::Regex;
    use spectral::assert_that;

    const SECRET_HASH: [u8; 32] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
        0, 1,
    ];

    const SECRET_HASH_REGEX: &str = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x00\x01";

    #[test]
    fn compiled_contract_is_same_length_as_template() {
        let htlc = Erc20Htlc::new(
            3_000_000,
            Address([0u8; 20]),
            Address([0u8; 20]),
            SECRET_HASH,
            Address([0u8; 20]),
            TokenQuantity([0u8; 32]),
        );

        assert_eq!(
            htlc.0.len(),
            CONTRACT_TEMPLATE.len(),
            "HTLC is the same length as template"
        );
    }

    #[test]
    fn given_input_data_when_compiled_should_contain_given_data() {
        let htlc = Erc20Htlc::new(
            2_000_000_000,
            Address([0u8; 20]),
            Address([0u8; 20]),
            SECRET_HASH,
            Address([0u8; 20]),
            TokenQuantity([0u8; 32]),
        );

        let compiled_code = htlc.0;

        // Allowed because `str::contains` (clippy's suggestion) does not apply to bytes
        // array
        #[allow(clippy::trivial_regex)]
        let _re_match = Regex::new(SECRET_HASH_REGEX)
            .expect("Could not create regex")
            .find(&compiled_code)
            .expect("Could not find secret hash in hex code");
    }

    #[test]
    fn test_replaced_placeholders_for_rfc_example() {
        let redeem_identity =
            <[u8; 20]>::from_hex("53fd2cac865d3aa1ad6fbdebaa00802c94239fba").unwrap();
        let refund_identity =
            <[u8; 20]>::from_hex("0f59e9e105be01d5e2206792a267406f255c5ea5").unwrap();
        let token_contract =
            <[u8; 20]>::from_hex("b97048628db6b661d4c2aa833e95dbe1a905b280").unwrap();
        let secret_hash = <[u8; 32]>::from_hex(
            "ac5a18da6431ed256965b873ef49dc15a70a0a66e2d28d0c226b5db040123727",
        )
        .unwrap();
        let token_quantity = <[u8; 32]>::from_hex(
            "0000000000000000000000000000000000000000000000000DE0B6B3A7640000",
        )
        .unwrap();
        let expiry = 1_552_263_040;

        let htlc = Erc20Htlc::new(
            expiry,
            Address(refund_identity),
            Address(redeem_identity),
            secret_hash,
            Address(token_contract),
            TokenQuantity(token_quantity),
        );

        let compiled_code = htlc.0;
        let contract_string = compiled_code.encode_hex::<String>();

        let expected_contract_code = "61018c61000f60003961018c6000f3361561007957602036141561004f57602060006000376020602160206000600060026048f17fac5a18da6431ed256965b873ef49dc15a70a0a66e2d28d0c226b5db04012372760215114166100ae575b7f696e76616c69645365637265740000000000000000000000000000000000000060005260206000fd5b42635c85a780106100f1577f746f6f4561726c7900000000000000000000000000000000000000000000000060005260206000fd5b7f72656465656d656400000000000000000000000000000000000000000000000060206000a17353fd2cac865d3aa1ad6fbdebaa00802c94239fba602052610134565b7f726566756e64656400000000000000000000000000000000000000000000000060006000a1730f59e9e105be01d5e2206792a267406f255c5ea5602052610134565b63a9059cbb6000527f0000000000000000000000000000000000000000000000000de0b6b3a7640000604052602060606044601c600073b97048628db6b661d4c2aa833e95dbe1a905b280620186a05a03f150602051ff";

        assert_that!(contract_string.as_str()).is_equal_to(expected_contract_code)
    }
}
