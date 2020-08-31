#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module};
use frame_system as system;

pub trait Trait: system::Trait {}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}


