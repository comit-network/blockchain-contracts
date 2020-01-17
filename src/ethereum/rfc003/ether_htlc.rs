use crate::ethereum::Address;
use crate::{EthereumTimestamp, FitIntoPlaceholderSlice, SecretHash};
use hex_literal::hex;

// contract template RFC: https://github.com/comit-network/RFCs/blob/master/RFC-007-SWAP-Basic-Ether.md#contract
pub const CONTRACT_TEMPLATE: [u8;315] = hex!("61012c61000f60003961012c6000f3361561005357602036141561008857602060006000376020602160206000600060026048f17f100000000000000000000000000000000000000000000000000000000000000160215114166100b257610088565b426320000002106100ef577f746f6f4561726c7900000000000000000000000000000000000000000000000060005260206000fd5b7f696e76616c69645365637265740000000000000000000000000000000000000060005260206000fd5b7f72656465656d656400000000000000000000000000000000000000000000000060206000a1733000000000000000000000000000000000000003ff5b7f726566756e64656400000000000000000000000000000000000000000000000060006000a1734000000000000000000000000000000000000004ff");

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
        EthereumTimestamp(expiry).fit_into_placeholder_slice(&mut contract[101..105]);
        redeem_identity.fit_into_placeholder_slice(&mut contract[233..253]);
        refund_identity.fit_into_placeholder_slice(&mut contract[294..314]);

        EtherHtlc(contract)
    }

    pub fn deployment_gas_limit(&self) -> u64 {
        121_800
    }

    pub fn tx_gas_limit() -> u64 {
        100_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::bytes::Regex;

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
}
