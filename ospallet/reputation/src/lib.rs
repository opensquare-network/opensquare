#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage};
use frame_system as system;

use crate::types::{Behavior, BountyRemarkCollaborationResult, BountyResolveCollaborationResult};

mod types;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as OsReputation {
        // TODO: change the value to BigInt
        pub BehaviorScore get(fn behavior_score): map hasher(blake2_128_concat) T::AccountId => i128;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}

impl<T: Trait> Module<T> {
    pub fn add_behavior_score(target: &T::AccountId, score: i128) {
        let pre_score = Self::behavior_score(target);
        // FIXME: Apply safe math
        BehaviorScore::<T>::insert(target, pre_score + score)
    }

    pub fn add_behavior_score_by_behavior(target: &T::AccountId, behavior: &Behavior) {
        let score = Self::get_behavior_score(behavior);
        Self::add_behavior_score(target, score)
    }

    pub fn get_behavior_score(behavior: &Behavior) -> i128 {
        return match behavior {
            Behavior::BountyResolve(BountyResolveCollaborationResult::Success) => 10,
            Behavior::BountyResolve(BountyResolveCollaborationResult::Fail) => -2,
            Behavior::BountyResolve(BountyResolveCollaborationResult::GiveUp) => -4,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Bad) => -2,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::NotGood) => 0,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Fine) => 1,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Good) => 3,
            Behavior::BountyRemark(BountyRemarkCollaborationResult::Perfect) => 5,
        };
    }
}
