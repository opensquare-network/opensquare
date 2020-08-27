#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use frame_support::{decl_module, decl_storage};
use frame_system as system;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as OsReputation {
        pub BehaviorScore get(fn behavior_score): map hasher(blake2_128_concat) T::AccountId => i128;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}
