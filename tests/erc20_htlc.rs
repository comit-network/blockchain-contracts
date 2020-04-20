#![warn(unused_extern_crates, missing_debug_implementations, rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod ethereum_helper;
pub mod htlc_harness;
pub mod parity_client;

use crate::{
    ethereum_helper::transaction::UnsignedTransaction,
    htlc_harness::{
        erc20_harness, sleep_until, CustomSizeSecret, Erc20HarnessParams, Timestamp, SECRET,
    },
};

use crate::parity_client::ParityClient;
use blockchain_contracts::ethereum::erc20_htlc::Erc20Htlc;
use blockchain_contracts::ethereum::Address;
use blockchain_contracts::ethereum::TokenQuantity;
use blockchain_contracts::ethereum::INVALID_SECRET;
use blockchain_contracts::ethereum::REDEEMED_LOG_MSG;
use blockchain_contracts::ethereum::REFUNDED_LOG_MSG;
use blockchain_contracts::ethereum::TOO_EARLY;
use serde_json::json;
use spectral::prelude::*;
use testcontainers::clients::Cli;
use web3::error::Error::Rpc;
use web3::types::{Bytes, TransactionReceipt, H256, U256};

#[test]
fn given_erc20_token_should_deploy_erc20_htlc_and_fund_htlc() {
    let docker = Cli::default();
    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(&docker, Erc20HarnessParams::default());

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(1000)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Fund erc20 htlc
    let tx = client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    let transaction_receipt = client.receipt(tx);
    log::debug!("used gas ERC20 fund {:?}", transaction_receipt.gas_used);

    // Check htlc funding
    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Send correct secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(SECRET.to_vec())),
        Erc20Htlc::redeem_tx_gas_limit().into(),
    );
    log::debug!("used gas ERC20 redeem {:?}", transaction_receipt.gas_used);

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(
        client.token_balance_of(token_contract, bob),
        U256::from(400)
    );
}

#[test]
fn given_funded_erc20_htlc_when_redeemed_with_secret_then_tokens_are_transferred() {
    let docker = Cli::default();
    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(&docker, Erc20HarnessParams::default());

    // fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Send correct secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(SECRET.to_vec())),
        U256::from(Erc20Htlc::redeem_tx_gas_limit()),
    );
    log::debug!("used gas ERC20 redeemed {:?}", transaction_receipt.gas_used);

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(
        client.token_balance_of(token_contract, bob),
        U256::from(400)
    );
}

#[test]
fn given_deployed_erc20_htlc_when_refunded_after_expiry_time_then_tokens_are_refunded() {
    let docker = Cli::default();
    let harness_params = Erc20HarnessParams::default();
    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(&docker, harness_params.clone());

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );

    // Wait for the contract to expire
    sleep_until(harness_params.htlc_refund_timestamp);
    let transaction_receipt =
        client.send_data(htlc_address, None, Erc20Htlc::refund_tx_gas_limit().into());
    log::debug!("used gas ERC20 refund {:?}", transaction_receipt.gas_used);

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(1000)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));
}

#[test]
fn given_deployed_erc20_htlc_when_expiry_time_not_yet_reached_should_revert_tx_with_error() {
    let docker = Cli::default();
    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(
            &docker,
            Erc20HarnessParams {
                htlc_refund_timestamp: Timestamp::now().plus(1_000_000),
                ..Default::default()
            },
        );

    // fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Don't wait for the timeout and don't send a secret
    let transaction_receipt =
        client.send_data(htlc_address, None, Erc20Htlc::refund_tx_gas_limit().into());
    log::debug!(
        "used gas ERC20 refund too early {:?}",
        transaction_receipt.gas_used
    );

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );

    assert_return_data(&client, transaction_receipt, TOO_EARLY);
}

#[test]
fn given_not_enough_tokens_when_redeemed_token_balances_dont_change() {
    let docker = Cli::default();
    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(
            &docker,
            Erc20HarnessParams {
                alice_initial_tokens: U256::from(200),
                ..Default::default()
            },
        );

    // fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(200)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Send correct secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(SECRET.to_vec())),
        Erc20Htlc::redeem_tx_gas_limit().into(),
    );
    log::debug!(
        "used gas ERC20 redeemed not enough token {:?}",
        transaction_receipt.gas_used
    );

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(200)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));
    assert_eq!(client.get_contract_code(htlc_address), Bytes::default());
}

#[test]
fn given_htlc_and_redeem_should_emit_redeem_log_msg_with_secret() {
    let docker = Cli::default();
    let (_alice, _bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(&docker, Erc20HarnessParams::default());

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    // Send correct secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(SECRET.to_vec())),
        500_000.into(), // This is for test purposes only
    );
    log::debug!("used gas ERC20 redeem {:?}", transaction_receipt.gas_used);

    // Should contain 2 logs: 1 for token transfer 1 for redeeming the htlc
    assert_that(&transaction_receipt.logs.len()).is_equal_to(2);
    assert_that(&transaction_receipt.logs[0].data).is_equal_to(Bytes(SECRET.to_vec()));

    let redeem_topic: H256 = REDEEMED_LOG_MSG.parse().unwrap();
    let refund_topic: H256 = REFUNDED_LOG_MSG.parse().unwrap();

    let topics: Vec<H256> = transaction_receipt
        .logs
        .into_iter()
        .flat_map(|s| s.topics)
        .collect();
    assert_that(&topics).has_length(4);
    assert_that(&topics).contains(redeem_topic);
    assert_that(&topics).does_not_contain(refund_topic);
}

#[test]
fn given_htlc_and_refund_should_emit_refund_log_msg() {
    let docker = Cli::default();
    let harness_params = Erc20HarnessParams::default();
    let (_alice, _bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(&docker, harness_params.clone());

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    // Wait for the contract to expire
    sleep_until(harness_params.htlc_refund_timestamp);
    // Send correct secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        None,
        500_000.into(), // This is for test purposes only
    );

    log::debug!("used gas ERC20 refund {:?}", transaction_receipt.gas_used);

    // Should contain 2 logs: 1 for token transfer 1 for redeeming the htlc
    assert_that(&transaction_receipt.logs.len()).is_equal_to(2);

    let redeem_topic: H256 = REDEEMED_LOG_MSG.parse().unwrap();
    let refund_topic: H256 = REFUNDED_LOG_MSG.parse().unwrap();

    let topics: Vec<H256> = transaction_receipt
        .clone()
        .logs
        .into_iter()
        .flat_map(|s| s.topics)
        .collect();
    assert_that(&topics).has_length(4);
    assert_that(&topics).does_not_contain(redeem_topic);
    assert_that(&topics).contains(refund_topic);
    assert_that(&transaction_receipt.logs[0].data).is_equal_to(Bytes(vec![]));
}

#[test]
fn given_funded_erc20_htlc_when_redeemed_with_short_secret_should_revert_with_error() {
    let docker = Cli::default();
    let secret = CustomSizeSecret(vec![
        1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ]);

    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(
            &docker,
            Erc20HarnessParams::default().with_secret_hash(secret.hash()),
        );

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Send short secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(vec![1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8])),
        Erc20Htlc::redeem_tx_gas_limit().into(),
    );

    log::debug!(
        "used gas ERC20 redeem short secret {:?}",
        transaction_receipt.gas_used
    );

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

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

    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(
            &docker,
            Erc20HarnessParams::default().with_secret_hash(secret.hash()),
        );

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Send short secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(secret_vec)),
        Erc20Htlc::redeem_tx_gas_limit().into(),
    );

    log::debug!("used gas ERC20 redeem {:?}", transaction_receipt.gas_used);

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(0)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(
        client.token_balance_of(token_contract, bob),
        U256::from(400)
    );
}

#[test]
fn given_short_zero_secret_htlc_should_revert_tx_with_error() {
    let docker = Cli::default();
    let secret = CustomSizeSecret(vec![
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ]);

    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(
            &docker,
            Erc20HarnessParams::default().with_secret_hash(secret.hash()),
        );

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    // Send short secret to contract
    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(vec![
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ])),
        Erc20Htlc::redeem_tx_gas_limit().into(),
    );

    log::debug!(
        "used gas ERC20 redeem zero secret {:?}",
        transaction_receipt.gas_used
    );

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    assert_return_data(&client, transaction_receipt, INVALID_SECRET);
}

#[test]
fn given_invalid_secret_htlc_should_revert_tx_with_error() {
    let docker = Cli::default();
    let secret_vec = vec![
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ];

    let (alice, bob, htlc_address, token_contract, token_amount, client, _handle, _container) =
        erc20_harness(&docker, Erc20HarnessParams::default());

    // Fund erc20 htlc
    client.sign_and_send(|nonce, gas_price| UnsignedTransaction {
        nonce,
        gas_price,
        gas_limit: U256::from(100_000),
        to: Some(token_contract),
        value: U256::from(0),
        data: Some(
            Erc20Htlc::transfer_erc20_tx_payload(
                TokenQuantity(token_amount.into()),
                Address(htlc_address.into()),
            )
            .into(),
        ),
    });

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(
        client.token_balance_of(token_contract, alice),
        U256::from(600)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

    let transaction_receipt = client.send_data(
        htlc_address,
        Some(Bytes(secret_vec)),
        500_000.into(), // This is for test purposes only
    );
    log::debug!(
        "used gas ERC20 invalid secret {:?}",
        transaction_receipt.gas_used
    );

    assert_eq!(
        client.token_balance_of(token_contract, htlc_address),
        U256::from(400)
    );
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));
    assert_eq!(client.token_balance_of(token_contract, bob), U256::from(0));

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
