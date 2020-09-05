#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{debug, decl_module, decl_storage, weights::Weight};
use frame_system as system;

use sp_runtime::traits::SaturatedConversion;

use frame_support::traits::Currency;

use crate::constants::DEFAULT_BLOCKS_PER_SESSION;
use crate::types::{MiningPower, SessionIndex};
pub use opensquare_primitives::CurrencyId;

mod constants;
mod types;

pub trait Trait: system::Trait {
    type Currency: Currency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OsMining {
        SessionAccountMiningPower get(fn session_account_mining_power):
            double_map hasher(identity) SessionIndex,
                hasher(blake2_128_concat) T::AccountId => MiningPower;
        SessionTotalMiningPower get(fn session_total_mining_power):
            map hasher(identity) SessionIndex => MiningPower;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            debug::info!("new block initialize, {:?}", now);
            let is_session_end = now.saturated_into::<u32>() % DEFAULT_BLOCKS_PER_SESSION == 0;
            if is_session_end {
                let total_issuance = T::Currency::total_issuance();
                let issuance = total_issuance.saturated_into::<u128>() / 100;

                debug::info!("new issuance, {:?}", issuance);
                // TODO: 1. generate jackpot address for this session
                // TODO: 2. depost the issuance to the jackpot address
            }

            10_00000
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn add_mining_power(target: &T::AccountId, power: MiningPower) {
        let now = <frame_system::Module<T>>::block_number();
        let session_index = now.saturated_into::<u32>() / DEFAULT_BLOCKS_PER_SESSION;

        SessionAccountMiningPower::<T>::mutate(&session_index, &target, |pre| {
            pre.checked_add(power)
        });
    }

    pub fn add_session_total_mining_power(power: MiningPower) {
        let now = <frame_system::Module<T>>::block_number();
        let session_index = now.saturated_into::<u32>() / DEFAULT_BLOCKS_PER_SESSION;

        SessionTotalMiningPower::mutate(&session_index, |pre| pre.checked_add(power));
    }
}
