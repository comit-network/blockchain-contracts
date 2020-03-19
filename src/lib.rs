#![warn(
    unused_extern_crates,
    missing_debug_implementations,
    rust_2018_idioms,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::fallible_impl_from,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::print_stdout,
    clippy::dbg_macro
)]
#![forbid(unsafe_code)]

pub mod bitcoin;
pub mod ethereum;
mod fit_into_placeholder_slice;

pub use self::fit_into_placeholder_slice::{
    EthereumTimestamp, FitIntoPlaceholderSlice, SecretHash,
};
