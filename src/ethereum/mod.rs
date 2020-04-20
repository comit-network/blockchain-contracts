pub mod erc20_htlc;
pub mod ether_htlc;

pub use erc20_htlc::*;
pub use ether_htlc::*;

/// The log message emitted when the HTLC is redeemed.
///
/// These are the hex-encoded ASCII-codepoints of the word "redeemed", padded to a length of 32 bytes.
pub const REDEEMED_LOG_MSG: &str =
    "72656465656d6564000000000000000000000000000000000000000000000000";

/// The log message emitted when the HTLC is refunded.
///
/// These are the hex-encoded ASCII-codepoints of the word "refund", padded to a length of 32 bytes.
pub const REFUNDED_LOG_MSG: &str =
    "726566756e646564000000000000000000000000000000000000000000000000";

/// The returned message when someone attempted to refund the HTLC before the timeout.
///
/// These are the hex-encoded ASCII-codepoints of the word "tooEarly", padded to a length of 32 bytes.
pub const TOO_EARLY: &str = "746f6f4561726c79000000000000000000000000000000000000000000000000";

/// The returned message when someone attempted to redeem the HTLC with an invalid secret.
///
/// These are the hex-encoded ASCII-codepoints of the word "invalidSecret", padded to a length of 32 bytes.
pub const INVALID_SECRET: &str = "696e76616c696453656372657400000000000000000000000000000000000000";

/// The log message emitted by an ERC-20 token contract when a transfer is done, as per the ERC-20 EIP.
///
/// This is the result of keccak('Transfer(address,address,uint256)')/
pub const ERC20_TRANSFER: &str = "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// Represent a ERC20 token quantity
/// The inner byte array is Big Endian
#[derive(Clone, Copy, Debug)]
pub struct TokenQuantity(pub [u8; 32]);

/// Represent an Ethereum Address
/// The inner byte array is Big Endian
#[derive(Clone, Copy, Debug)]
pub struct Address(pub [u8; 20]);
