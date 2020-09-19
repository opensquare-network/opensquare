#![cfg_attr(not(feature = "std"), no_std)]

mod macros;
#[cfg(feature = "std")]
mod serde;

pub use self::macros::*;
#[cfg(feature = "std")]
pub use self::serde::{serde_hex, serde_text};

use frame_support::dispatch::{DispatchError, DispatchResult};

/// Although xss is imperceptible on-chain, we merely want to make it look safer off-chain.
#[inline]
pub fn xss_check(input: &[u8]) -> DispatchResult {
    if input.contains(&b'<') || input.contains(&b'>') {
        Err(DispatchError::Other(
            "'<' and '>' are not allowed, which could be abused off-chain.",
        ))?;
    }
    Ok(())
}
