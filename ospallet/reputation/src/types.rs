#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "std")]
#[derive(Serialize, Deserialize)]
enum BountyResolveCollaborationResult {
    Success,
    Fail,
    GiveUp,
}

#[cfg(feature = "std")]
#[derive(Serialize, Deserialize)]
enum BountyRemarkCollaborationResult {
    Bad,
    NotGood,
    Fine,
    Good,
    Perfect,
}

#[cfg(feature = "std")]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CollaborationResult {
    BountyResolve(BountyResolveCollaborationResult),
    BountyRemark(BountyRemarkCollaborationResult),
}

#[cfg(feature = "std")]
#[allow(dead_code)]
type Behavior = CollaborationResult;
