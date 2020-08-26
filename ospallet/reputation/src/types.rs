#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
enum BountyResolveCollaborationResult {
    Success,
    Fail,
    GiveUp,
}

#[allow(dead_code)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
enum BountyRemarkCollaborationResult {
    Bad,
    NotGood,
    Fine,
    Good,
    Perfect,
}

#[allow(dead_code)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize), serde(untagged))]
enum CollaborationResult {
    BountyResolve(BountyResolveCollaborationResult),
    BountyRemark(BountyRemarkCollaborationResult),
}

#[cfg(feature = "std")]
#[allow(dead_code)]
type Behavior = CollaborationResult;
