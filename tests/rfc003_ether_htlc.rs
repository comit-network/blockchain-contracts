#![warn(unused_extern_crates, missing_debug_implementations, rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod ethereum_helper;
pub mod htlc_harness;
pub mod parity_client;

use crate::htlc_harness::{
    ether_harness, sleep_until, CustomSizeSecret, EtherHarnessParams, Timestamp, SECRET,
};
use parity_client::ParityClient;
use serde_json::json;
use spectral::prelude::*;
use testcontainers::clients::Cli;
use web3::error::Error::Rpc;
use web3::types::{Bytes, TransactionReceipt, H256, U256};

// keccak256(Redeemed())
const REDEEMED_LOG_MSG: &str = "B8CAC300E37F03AD332E581DEA21B2F0B84EAAADC184A295FEF71E81F44A7413";
// keccak256(Refunded())
const REFUNDED_LOG_MSG: &str = "5D26862916391BF49478B2F5103B0720A842B45EF145A268F2CD1FB2AED55178";
const TOOEARLY_LOG_MSG: &str = "0xbbad9d5bf43fc68b6ab3d56342306bfc459abe19dd1d361dbcab75c00400b85c";
const WRONGSECRET_LOG_MSG: &str =
    "0x05f03ebf077f616c9d02b91c7fcbac32beef85527aedff9cf81357a5a00c8c41";

#[test]
fn given_deployed_htlc_when_redeemed_with_secret_then_money_is_transferred() {
    let docker = Cli::default();
    let (_alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    // Send correct secret to contract
    let transaction_receipt = client.send_data(htlc, Some(Bytes(SECRET.to_vec())));
    log::debug!("used gas ETH redeem {:?}", transaction_receipt.gas_used);

    assert_eq!(
        client.eth_balance_of(bob),
        U256::from("0400000000000000000")
    );
    assert_eq!(client.eth_balance_of(htlc), U256::from(0));

    assert_that(&transaction_receipt.logs).has_length(1);
    let topic: H256 = REDEEMED_LOG_MSG.parse().unwrap();
    assert_that(&transaction_receipt.logs[0].topics).has_length(1);
    assert_that(&transaction_receipt.logs[0].topics).contains(topic);
    assert_that(&transaction_receipt.logs[0].data).is_equal_to(Bytes(SECRET.to_vec()));
}

#[test]
fn given_deployed_htlc_when_refunded_after_expiry_time_then_money_is_refunded() {
    let docker = Cli::default();
    let harness_params = EtherHarnessParams::default();
    let (_alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, harness_params.clone());

    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    // Wait for the contract to expire
    sleep_until(harness_params.htlc_refund_timestamp);
    let transaction_receipt = client.send_data(htlc, None);
    log::debug!("used gas ETH refund {:?}", transaction_receipt.gas_used);

    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(client.eth_balance_of(htlc), U256::from(0));
}

#[test]
fn given_deployed_htlc_when_refunded_too_early_should_revert_tx_with_error() {
    let docker = Cli::default();
    let harness_params = EtherHarnessParams {
        htlc_refund_timestamp: Timestamp::now().plus(1_000_000),
        ..Default::default()
    };
    let (_alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, harness_params.clone());

    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    // Don't wait for the timeout and don't send a secret
    let transaction_receipt = client.send_data(htlc, None);
    log::debug!("used gas ETH too early {:?}", transaction_receipt.gas_used);

    // Check refund did not happen
    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );
    assert_return_data(&client, transaction_receipt, TOOEARLY_LOG_MSG);
}

#[test]
fn given_htlc_and_redeem_should_emit_redeem_log_msg_with_secret() {
    let docker = Cli::default();
    let (_alice, _bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    // Send correct secret to contract
    let transaction_receipt = client.send_data(htlc, Some(Bytes(SECRET.to_vec())));
    log::debug!("used gas ETH redeem {:?}", transaction_receipt.gas_used);
}

#[test]
fn given_htlc_and_refund_should_emit_refund_log_msg() {
    let docker = Cli::default();
    let harness_params = EtherHarnessParams::default();
    let (_alice, _bob, htlc, client, _handle, _container) =
        ether_harness(&docker, harness_params.clone());

    // Wait for the timelock to expire
    sleep_until(harness_params.htlc_refund_timestamp);
    let transaction_receipt = client.send_data(htlc, None);

    assert_that(&transaction_receipt.logs).has_length(1);
    let topic: H256 = REFUNDED_LOG_MSG.parse().unwrap();
    assert_that(&transaction_receipt.logs[0].topics).has_length(1);
    assert_that(&transaction_receipt.logs[0].topics).contains(topic);
    assert_that(&transaction_receipt.logs[0].data).is_equal_to(Bytes(vec![]));
}

#[test]
fn given_deployed_htlc_when_redeem_with_short_secret_should_revert_with_error() {
    let docker = Cli::default();
    let secret = CustomSizeSecret(vec![
        1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ]);

    let (_alice, bob, htlc, client, _handle, _container) = ether_harness(
        &docker,
        EtherHarnessParams::default().with_secret_hash(secret.hash()),
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    let transaction_receipt = client.send_data(
        htlc,
        Some(Bytes(vec![1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8])),
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    assert_return_data(&client, transaction_receipt, WRONGSECRET_LOG_MSG);
}

#[test]
fn given_correct_zero_secret_htlc_should_redeem() {
    let docker = Cli::default();
    let secret_vec = vec![
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ];
    let secret = CustomSizeSecret(secret_vec.clone());

    let (_alice, bob, htlc, client, _handle, _container) = ether_harness(
        &docker,
        EtherHarnessParams::default().with_secret_hash(secret.hash()),
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    client.send_data(htlc, Some(Bytes(secret_vec)));

    assert_eq!(
        client.eth_balance_of(bob),
        U256::from("0400000000000000000")
    );

    assert_eq!(client.eth_balance_of(htlc), U256::from(0));
}

#[test]
fn given_short_zero_secret_htlc_should_revert_tx_with_error() {
    let docker = Cli::default();
    let secret = CustomSizeSecret(vec![
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ]);

    let (_alice, bob, htlc, client, _handle, _container) = ether_harness(
        &docker,
        EtherHarnessParams::default().with_secret_hash(secret.hash()),
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    let transaction_receipt = client.send_data(
        htlc,
        Some(Bytes(vec![
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ])),
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );
    assert_return_data(&client, transaction_receipt, WRONGSECRET_LOG_MSG);
}

#[test]
fn given_invalid_secret_htlc_should_revert_tx_with_error() {
    let docker = Cli::default();
    let (_alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    // Send incorrect secret to contract
    // Send incorrect secret to contract
    let transaction_receipt = client.send_data(htlc, Some(Bytes(b"I'm a h4x0r".to_vec())));
    log::debug!(
        "used gas ETH invalid secret {:?}",
        transaction_receipt.gas_used
    );

    // Check redeem did not happen
    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );
    assert_return_data(&client, transaction_receipt, WRONGSECRET_LOG_MSG);
}

fn assert_return_data(
    client: &ParityClient,
    transaction_receipt: TransactionReceipt,
    error_code: &str,
) {
    let result = client.get_return_data(transaction_receipt);
    let return_data = result.err().unwrap();
    let json = json!(format!("{}{}", "Reverted ", error_code));
    match return_data {
        Rpc(e) => {
            asserting(&"contains VM message")
                .that(&e.message)
                .contains("VM execution error.");
            asserting(&"contains revert reason")
                .that(&e.data.unwrap())
                .is_equal_to(json);
        }
        _ => assert_that(&true).is_false(),
    };
}
