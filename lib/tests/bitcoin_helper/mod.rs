#![warn(unused_extern_crates, missing_debug_implementations)]
#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use rust_bitcoin::hashes::hex::FromHex;
use rust_bitcoin::Txid;
use rust_bitcoin::{
    consensus::deserialize, hashes::sha256d, Address, Amount, Network, OutPoint, PublicKey, Script,
    Transaction, TxOut,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use testcontainers::images::coblox_bitcoincore::RpcAuth;
use testcontainers::{images::coblox_bitcoincore::BitcoinCore, Container, Docker};

#[derive(serde::Serialize)]
struct JsonRpcRequest<T> {
    id: String,
    jsonrpc: String,
    method: String,
    params: T,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<R> {
    result: Option<R>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RPC failed with: {}", self.message)
    }
}

impl std::error::Error for RpcError {}

impl<T> JsonRpcRequest<T> {
    fn new(method: &str, params: T) -> Self {
        Self {
            id: "test".to_owned(),
            jsonrpc: "1.0".to_owned(),
            method: method.to_owned(),
            params,
        }
    }
}

fn serialize<T: Serialize>(t: T) -> Result<serde_json::Value> {
    let value = serde_json::to_value(t).context("failed to serialize value")?;

    Ok(value)
}

#[derive(Debug)]
pub struct Client {
    endpoint: String,
    auth: RpcAuth,
}

impl Client {
    pub fn new(endpoint: String, auth: RpcAuth) -> Client {
        Client { endpoint, auth }
    }

    pub fn get_new_address(&self) -> Result<Address> {
        let request = JsonRpcRequest::<Vec<()>>::new("getnewaddress", Vec::new());

        Ok(reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .json::<JsonRpcResponse<_>>()?
            .result
            .expect("getnewaddress response result is null"))
    }

    pub fn generate(&self, num: u32) -> Result<()> {
        let request = JsonRpcRequest::new("generate", vec![serialize(num)?]);

        let _ = reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .text()?;
        Ok(())
    }

    pub fn send_raw_transaction(&self, hex: String) -> Result<sha256d::Hash> {
        let request = JsonRpcRequest::new("sendrawtransaction", vec![serialize(hex)?]);

        let response = reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .json()?;

        match response {
            JsonRpcResponse {
                result: None,
                error: Some(error),
            } => Err(error.into()),
            JsonRpcResponse {
                result: Some(result),
                error: None,
            } => Ok(result),
            _ => panic!("Received response with both result and error null"),
        }
    }

    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction> {
        let request = JsonRpcRequest::new("getrawtransaction", vec![serialize(txid)?]);

        let response: JsonRpcResponse<String> = reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .json()?;

        Ok(deserialize(&Vec::<u8>::from_hex(
            &response
                .result
                .expect("getrawtransaction response result is null"),
        )?)?)
    }

    pub fn get_blockchain_info(&self) -> Result<BlockchainInfo> {
        let request = JsonRpcRequest::<Vec<()>>::new("getblockchaininfo", vec![]);

        Ok(reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .json::<JsonRpcResponse<_>>()?
            .result
            .expect("getblockchaininfo response result is null"))
    }

    pub fn list_unspent(&self, addresses: Option<&[Address]>) -> Result<Vec<Unspent>> {
        let request = JsonRpcRequest::new(
            "listunspent",
            vec![
                serde_json::Value::Null,
                serde_json::Value::Null,
                serialize(addresses)?,
            ],
        );

        Ok(reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .json::<JsonRpcResponse<_>>()?
            .result
            .expect("list_unspent response result is null"))
    }

    pub fn send_to_address(&self, address: &Address, amount: Amount) -> Result<Txid> {
        let request = JsonRpcRequest::new(
            "sendtoaddress",
            vec![serialize(address)?, serialize(amount.as_btc())?],
        );

        Ok(reqwest::blocking::Client::new()
            .post(self.endpoint.as_str())
            .basic_auth(&self.auth.username, Some(&self.auth.password))
            .json(&request)
            .send()?
            .json::<JsonRpcResponse<_>>()?
            .result
            .expect("sendtoaddress response result is null"))
    }

    pub fn find_utxo_at_tx_for_address(
        &self,
        txid: &sha256d::Hash,
        address: &Address,
    ) -> Option<TxOut> {
        let address = address.clone();
        let unspent = self.list_unspent(Some(&[address])).unwrap();

        #[allow(clippy::cast_sign_loss)] // it is just for the tests
        unspent
            .into_iter()
            .find(|utxo| utxo.txid == *txid)
            .map(|result| TxOut {
                value: Amount::from_btc(result.amount)
                    .expect("Could not convert received amount to Amount")
                    .as_sat(),
                script_pubkey: result.script_pub_key,
            })
    }

    pub fn find_vout_for_address(&self, txid: &Txid, address: &Address) -> OutPoint {
        let tx = self.get_raw_transaction(&txid).unwrap();

        tx.output
            .iter()
            .enumerate()
            .find_map(|(vout, txout)| {
                let vout = u32::try_from(vout).unwrap();
                if txout.script_pubkey == address.script_pubkey() {
                    Some(OutPoint { txid: *txid, vout })
                } else {
                    None
                }
            })
            .unwrap()
    }

    pub fn mine_bitcoins(&self) {
        self.generate(101).unwrap();
    }

    pub fn create_p2wpkh_vout_at(
        &self,
        public_key: rust_bitcoin::secp256k1::PublicKey,
        amount: Amount,
    ) -> (Txid, OutPoint) {
        let address = Address::p2wpkh(
            &PublicKey {
                compressed: true,
                key: public_key,
            },
            Network::Regtest,
        )
        .unwrap();

        let txid = self.send_to_address(&address, amount).unwrap();

        self.generate(1).unwrap();

        let vout = self.find_vout_for_address(&txid, &address);

        (txid, vout)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unspent {
    pub txid: sha256d::Hash,
    pub vout: u32,
    pub address: Option<Address>,
    pub amount: f64,
    pub script_pub_key: Script,
}

#[derive(Debug, Deserialize)]
pub struct BlockchainInfo {
    pub mediantime: u64,
}

pub fn new_tc_bitcoincore_client<D: Docker>(container: &Container<'_, D, BitcoinCore>) -> Client {
    let port = container.get_host_port(18443).unwrap();
    let auth = container.image().auth();

    let endpoint = format!("http://localhost:{}", port);

    Client::new(endpoint, auth.to_owned())
}
