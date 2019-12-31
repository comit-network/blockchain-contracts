pub mod rfc003;

/// Represent a ERC20 token quantity
/// The inner byte array is Big Endian
#[derive(Debug)]
pub struct TokenQuantity(pub [u8; 32]);

/// Represent an Ethereum Address
/// The inner byte array is Big Endian
#[derive(Debug)]
pub struct Address(pub [u8; 20]);
