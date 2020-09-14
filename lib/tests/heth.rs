#![warn(unused_extern_crates, missing_debug_implementations, rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod ethereum_helper;
pub mod htlc_harness;
pub mod parity_client;

use crate::htlc_harness::{
    ether_harness, sleep_until, CustomSizeSecret, EtherHarnessParams, Timestamp, SECRET,
};
use blockchain_contracts::ethereum::REDEEMED_LOG_MSG;
use blockchain_contracts::ethereum::REFUNDED_LOG_MSG;
use blockchain_contracts::ethereum::TOO_EARLY;
use blockchain_contracts::ethereum::{heth::Htlc, INVALID_SECRET};
use parity_client::ParityClient;
use serde_json::json;
use spectral::prelude::*;
use testcontainers::clients::Cli;
use web3::error::Error::Rpc;
use web3::types::{Bytes, Log, TransactionReceipt, H256, U256};

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
    let transaction_receipt = client.send_data(
        htlc,
        Some(Bytes(SECRET.to_vec())),
        U256::from(Htlc::redeem_tx_gas_limit()),
    );
    log::debug!("used gas ETH redeem {:?}", transaction_receipt.gas_used);

    assert_eq!(
        client.eth_balance_of(bob),
        U256::from("0400000000000000000")
    );
    assert_eq!(client.eth_balance_of(htlc), U256::from(0));

    let topic: H256 = REDEEMED_LOG_MSG.parse().unwrap();
    let Log { topics, data, .. } = &transaction_receipt.logs[0];

    assert_that(topics).contains(topic);
    assert_that(data).is_equal_to(Bytes(SECRET.to_vec()));
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
    let transaction_receipt = client.send_data(htlc, None, U256::from(Htlc::refund_tx_gas_limit()));
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
    let (_alice, bob, htlc, client, _handle, _container) = ether_harness(&docker, harness_params);

    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    // Don't wait for the timeout and don't send a secret
    let transaction_receipt = client.send_data(htlc, None, U256::from(Htlc::refund_tx_gas_limit()));
    log::debug!("used gas ETH too early {:?}", transaction_receipt.gas_used);

    // Check refund did not happen
    assert_eq!(client.eth_balance_of(bob), U256::from(0));
    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );
    assert_return_data(&client, transaction_receipt, TOO_EARLY);
}

#[test]
fn given_htlc_and_redeem_should_emit_redeem_log_msg_with_secret() {
    let docker = Cli::default();
    let (_alice, _bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    // Send correct secret to contract
    let transaction_receipt = client.send_data(
        htlc,
        Some(Bytes(SECRET.to_vec())),
        Htlc::redeem_tx_gas_limit().into(),
    );
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
    let transaction_receipt = client.send_data(htlc, None, Htlc::refund_tx_gas_limit().into());

    let topic: H256 = REFUNDED_LOG_MSG.parse().unwrap();
    let Log { topics, data, .. } = assert_that(&transaction_receipt.logs)
        .map(|x| &x[0])
        .subject;

    assert_that(topics).contains(topic);
    assert_that(data).is_equal_to(Bytes(vec![]));
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
        500_000.into(), // This is for test purposes only
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );

    assert_return_data(&client, transaction_receipt, INVALID_SECRET);
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

    client.send_data(
        htlc,
        Some(Bytes(secret_vec)),
        Htlc::redeem_tx_gas_limit().into(),
    );

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
        500_000.into(), // This is for test purposes only
    );

    assert_eq!(client.eth_balance_of(bob), U256::from(0));

    assert_eq!(
        client.eth_balance_of(htlc),
        U256::from("0400000000000000000")
    );
    assert_return_data(&client, transaction_receipt, INVALID_SECRET);
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
    let transaction_receipt = client.send_data(
        htlc,
        Some(Bytes(b"I'm a h4x0r".to_vec())),
        500_000.into(), // This is for test purposes only
    );
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
    assert_return_data(&client, transaction_receipt, INVALID_SECRET);
}

fn assert_return_data(
    client: &ParityClient,
    transaction_receipt: TransactionReceipt,
    error_code: &str,
) {
    let result = client.get_return_data(transaction_receipt);
    let return_data = result.err().unwrap();
    let json = json!(format!("{}0x{}", "Reverted ", error_code));
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
