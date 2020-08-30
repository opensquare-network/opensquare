use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use sp_runtime::RuntimeDebug;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BountyResolveCollaborationResult {
    Success,
    Fail,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BountyRemarkCollaborationResult {
    Bad,
    NotGood,
    Fine,
    Good,
    Perfect,
}

// Behavior represent the general collaboration result
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(untagged))]
pub enum Behavior {
    BountyResolve(BountyResolveCollaborationResult),
    BountyRemark(BountyRemarkCollaborationResult),
}

pub trait ReputationBuilder<AccountId> {
    fn add_behavior_score_by_behavior(target: &AccountId, behavior: &Behavior);

    fn get_behavior_score(behavior: &Behavior) -> i128;
}
