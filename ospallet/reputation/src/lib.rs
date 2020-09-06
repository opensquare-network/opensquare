#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_event, decl_storage};
use frame_system as system;

pub use crate::types::{
    Behavior, BountyRemarkCollaborationResult, BountyResolveCollaborationResult, ReputationBuilder,
};

mod types;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
    {
        ReputationAdded(AccountId, i128),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as OsReputation {
        pub BehaviorScore get(fn behavior_score): map hasher(blake2_128_concat) T::AccountId => i128;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {
    pub fn add_behavior_score(target: &T::AccountId, score: i128) {
        BehaviorScore::<T>::mutate(target, |pre| {
            if let Some(new_score) = pre.checked_add(score) {
                *pre = new_score;
                Self::deposit_event(
                    RawEvent::ReputationAdded(target.clone(), new_score)
                );
            }
        });
    }
}

impl<T: Trait> ReputationBuilder<T::AccountId> for Module<T> {
    // TODO: calc behavior score separately for funder and hunter
    fn add_behavior_score_by_behavior(target: &T::AccountId, behavior: &Behavior) {
        let score = Self::get_behavior_score(behavior);
        Self::add_behavior_score(target, score)
    }

    fn get_behavior_score(behavior: &Behavior) -> i128 {
        return match behavior {
            Behavior::BountyResolve(BountyResolveCollaborationResult::Success) => 10,
            Behavior::BountyResolve(BountyResolveCollaborationResult::Fail) => -2,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Bad) => -2,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::NotGood) => 0,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Fine) => 1,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Good) => 3,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Perfect) => 5,
        };
    }
}
