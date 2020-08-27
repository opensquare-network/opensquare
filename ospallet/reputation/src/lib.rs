#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage};
use frame_system as system;

use crate::types::{Behavior, BountyRemarkCollaborationResult, BountyResolveCollaborationResult};

mod types;

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

impl<T: Trait> Module<T> {
    pub fn get_behavior_score(behavior: Behavior) -> i128 {
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
