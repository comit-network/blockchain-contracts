use rust_bitcoin::{
    hashes::{hash160, Hash},
    secp256k1::{self, PublicKey, Secp256k1, SecretKey},
};
use std::{
    convert::TryFrom,
    fmt::{self, Display},
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PubkeyHash(hash160::Hash);

impl PubkeyHash {
    #[allow(dead_code)] // Only used in tests at the moment
    fn new<C: secp256k1::Signing>(secp: &Secp256k1<C>, secret_key: &SecretKey) -> Self {
        secp256k1::PublicKey::from_secret_key(secp, secret_key).into()
    }
}

impl From<hash160::Hash> for PubkeyHash {
    fn from(hash: hash160::Hash) -> PubkeyHash {
        PubkeyHash(hash)
    }
}

impl From<PublicKey> for PubkeyHash {
    fn from(public_key: PublicKey) -> PubkeyHash {
        PubkeyHash(
            <rust_bitcoin::hashes::hash160::Hash as rust_bitcoin::hashes::Hash>::hash(
                &public_key.serialize(),
            ),
        )
    }
}

impl<'a> TryFrom<&'a [u8]> for PubkeyHash {
    type Error = rust_bitcoin::hashes::error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(PubkeyHash(hash160::Hash::from_slice(value)?))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FromHexError {
    HashConversion(rust_bitcoin::hashes::error::Error),
}

impl From<rust_bitcoin::hashes::error::Error> for FromHexError {
    fn from(err: rust_bitcoin::hashes::error::Error) -> Self {
        FromHexError::HashConversion(err)
    }
}

impl Display for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", &self)
    }
}

impl AsRef<[u8]> for PubkeyHash {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Into<hash160::Hash> for PubkeyHash {
    fn into(self) -> hash160::Hash {
        self.0
    }
}

impl fmt::LowerHex for PubkeyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(format!("{:?}", self.0).as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rust_bitcoin::PrivateKey;
    use std::str::FromStr;

    #[test]
    fn correct_pubkeyhash_from_private_key() {
        let secp = Secp256k1::signing_only();

        let private_key =
            PrivateKey::from_str("L253jooDhCtNXJ7nVKy7ijtns7vU4nY49bYWqUH8R9qUAUZt87of").unwrap();
        let pubkey_hash = PubkeyHash::new(&secp, &private_key.key);

        assert_eq!(
            pubkey_hash,
            PubkeyHash::try_from(
                &hex::decode("8bc513e458372a3b3bb05818d09550295ce15949").unwrap()[..]
            )
            .unwrap()
        )
    }
}
