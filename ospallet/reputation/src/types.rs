#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BountyResolveCollaborationResult {
    Success,
    Fail,
    GiveUp,
}

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
