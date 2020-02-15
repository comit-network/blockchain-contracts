use crate::ethereum::Address;
use crate::{EthereumTimestamp, FitIntoPlaceholderSlice, SecretHash};
use hex_literal::hex;

// contract template RFC: https://github.com/comit-network/RFCs/blob/master/RFC-007-SWAP-Basic-Ether.md#contract
pub const CONTRACT_TEMPLATE: [u8;311] = hex!("61012861000f6000396101286000f3361561007957602036141561004f57602060006000376020602160206000600060026048f17f100000000000000000000000000000000000000000000000000000000000000160215114166100ae575b7f696e76616c69645365637265740000000000000000000000000000000000000060005260206000fd5b426320000002106100eb577f746f6f4561726c7900000000000000000000000000000000000000000000000060005260206000fd5b7f72656465656d656400000000000000000000000000000000000000000000000060206000a1733000000000000000000000000000000000000003ff5b7f726566756e64656400000000000000000000000000000000000000000000000060006000a1734000000000000000000000000000000000000004ff");

#[derive(Debug)]
pub struct EtherHtlc(Vec<u8>);

impl From<EtherHtlc> for Vec<u8> {
    fn from(htlc: EtherHtlc) -> Self {
        htlc.0
    }
}

impl EtherHtlc {
    pub fn new(
        expiry: u32,
        refund_identity: Address,
        redeem_identity: Address,
        secret_hash: [u8; 32],
    ) -> Self {
        let mut contract = CONTRACT_TEMPLATE.to_vec();
        SecretHash(secret_hash).fit_into_placeholder_slice(&mut contract[53..85]);
        EthereumTimestamp(expiry).fit_into_placeholder_slice(&mut contract[139..143]);
        redeem_identity.fit_into_placeholder_slice(&mut contract[229..249]);
        refund_identity.fit_into_placeholder_slice(&mut contract[290..310]);

        EtherHtlc(contract)
    }

    pub fn deploy_tx_gas_limit() -> u64 {
        // 126_386 to 126_450 consumed in local test
        130_000
    }

    pub fn redeem_tx_gas_limit() -> u64 {
        // 31_082 consumed in local test for successful redeeming
        // 21_809 consumed in local test for failed redeeming
        31082 + 24000
    }

    pub fn refund_tx_gas_limit() -> u64 {
        // 13_402 consumed in local test for successful refunding
        // 21_058 consumed in local test for failed refunding
        100_000
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
        let htlc = EtherHtlc::new(
            3_000_000,
            Address([0u8; 20]),
            Address([0u8; 20]),
            SECRET_HASH,
        );

        assert_eq!(
            htlc.0.len(),
            CONTRACT_TEMPLATE.len(),
            "HTLC is the same length as template"
        );
    }

    #[test]
    fn given_input_data_when_compiled_should_contain_given_data() {
        let htlc = EtherHtlc::new(
            2_000_000_000,
            Address([0u8; 20]),
            Address([0u8; 20]),
            SECRET_HASH,
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
        let secret_hash = <[u8; 32]>::from_hex(
            "ac5a18da6431ed256965b873ef49dc15a70a0a66e2d28d0c226b5db040123727",
        )
        .unwrap();
        let expiry = 1_552_263_040;

        let htlc = EtherHtlc::new(
            expiry,
            Address(refund_identity),
            Address(redeem_identity),
            secret_hash,
        );

        let compiled_code = htlc.0;
        let contract_string = compiled_code.encode_hex::<String>();

        let expected_contract_code = "61012861000f6000396101286000f3361561007957602036141561004f57602060006000376020602160206000600060026048f17fac5a18da6431ed256965b873ef49dc15a70a0a66e2d28d0c226b5db04012372760215114166100ae575b7f696e76616c69645365637265740000000000000000000000000000000000000060005260206000fd5b42635c85a780106100eb577f746f6f4561726c7900000000000000000000000000000000000000000000000060005260206000fd5b7f72656465656d656400000000000000000000000000000000000000000000000060206000a17353fd2cac865d3aa1ad6fbdebaa00802c94239fbaff5b7f726566756e64656400000000000000000000000000000000000000000000000060006000a1730f59e9e105be01d5e2206792a267406f255c5ea5ff";

        assert_that!(contract_string.as_str()).is_equal_to(expected_contract_code)
    }
}
