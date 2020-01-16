use crate::ethereum::Address;
use crate::{EthereumTimestamp, FitIntoPlaceholderSlice, SecretHash};
use hex_literal::hex;

// contract template RFC: https://github.com/comit-network/RFCs/blob/master/RFC-007-SWAP-Basic-Ether.md#contract
pub const CONTRACT_TEMPLATE: [u8;320] = hex!("61013161000f6000396101316000f3361561007a5760203614156100b157602060006000376020602160206000600060026048f17f100000000000000000000000000000000000000000000000000000000000000160215114166100b7577f05f03ebf077f616c9d02b91c7fcbac32beef85527aedff9cf81357a5a00c8c4160006000a160006000f35b426320000002106100f4577fbbad9d5bf43fc68b6ab3d56342306bfc459abe19dd1d361dbcab75c00400b85c60006000a160006000f35b60006000f35b7fb8cac300e37f03ad332e581dea21b2f0b84eaaadc184a295fef71e81f44a741360206000a1733000000000000000000000000000000000000003ff5b7f5d26862916391bf49478b2f5103b0720a842b45ef145a268f2cd1fb2aed5517860006000a1734000000000000000000000000000000000000004ff");

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
        EthereumTimestamp(expiry).fit_into_placeholder_slice(&mut contract[140..144]);
        refund_identity.fit_into_placeholder_slice(&mut contract[299..319]);
        redeem_identity.fit_into_placeholder_slice(&mut contract[238..258]);
        SecretHash(secret_hash).fit_into_placeholder_slice(&mut contract[53..85]);

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
