#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, weights::Weight,
};
use frame_system as system;
use frame_system::ensure_signed;

use sp_runtime::traits::SaturatedConversion;

use frame_support::traits::Currency;

use crate::constants::DEFAULT_BLOCKS_PER_SESSION;
use crate::types::{MiningPower, SessionIndex};
pub use opensquare_primitives::CurrencyId;

mod constants;
mod types;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>
    {
        SessionTotalRewardSet(SessionIndex, Balance),
        SessionTotalMiningPowerSet(SessionIndex, MiningPower),
        AccountMiningPowerSet(AccountId, SessionIndex, MiningPower),
        RewardClaimed(AccountId, SessionIndex, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as OsMining {
        SessionAccountMiningPower get(fn session_account_mining_power):
            double_map hasher(identity) SessionIndex,
                hasher(blake2_128_concat) T::AccountId => MiningPower;

        SessionTotalMiningPower get(fn session_total_mining_power):
            map hasher(identity) SessionIndex => MiningPower;

        SessionTotalReward get(fn session_total_reward):
            map hasher(identity) SessionIndex => BalanceOf<T>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 0]
        fn claim(origin, session_index: SessionIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let power = SessionAccountMiningPower::<T>::take(session_index, &who);
            if power > 0 {
                let total_power = Self::session_total_mining_power(session_index);

                let now = <frame_system::Module<T>>::block_number();
                let session_index = now.saturated_into::<u32>() / DEFAULT_BLOCKS_PER_SESSION;

                let total_reward = Self::session_total_reward(session_index);
                let reward = power.saturated_into::<BalanceOf<T>>() / total_power.saturated_into() * total_reward;
                T::Currency::deposit_creating(&who, reward);

                Self::deposit_event(RawEvent::RewardClaimed(who.clone(), session_index, reward));
            }

            Ok(())
        }

        fn on_initialize(now: T::BlockNumber) -> Weight {
            let is_session_end = now.saturated_into::<u32>() % DEFAULT_BLOCKS_PER_SESSION == 0;
            let total_issuance = T::Currency::total_issuance();

            if is_session_end {
                let issuance = total_issuance / 100.into();
                let session_index = now.saturated_into::<u32>() / DEFAULT_BLOCKS_PER_SESSION;
                SessionTotalReward::<T>::insert(session_index, issuance);

                Self::deposit_event(RawEvent::SessionTotalRewardSet(session_index, issuance));
            }

            10_00000
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn add_mining_power(target: &T::AccountId, power: MiningPower) {
        let now = <frame_system::Module<T>>::block_number();
        let session_index = now.saturated_into::<u32>() / DEFAULT_BLOCKS_PER_SESSION;

        SessionAccountMiningPower::<T>::mutate(session_index, &target, |pre| {
            *pre = pre.saturating_add(power);
            Self::deposit_event(RawEvent::AccountMiningPowerSet(
                target.clone(),
                session_index,
                *pre,
            ));
        });
    }

    pub fn add_session_total_mining_power(power: MiningPower) {
        let now = <frame_system::Module<T>>::block_number();
        let session_index = now.saturated_into::<u32>() / DEFAULT_BLOCKS_PER_SESSION;

        SessionTotalMiningPower::mutate(session_index, |pre| {
            *pre = pre.saturating_add(power);
            Self::deposit_event(RawEvent::SessionTotalMiningPowerSet(session_index, *pre));
        });
    }
}
