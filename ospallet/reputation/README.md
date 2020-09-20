# Reputation Module

## Terminology

- CollaborationUnit: A interaction of between roles of OpenSquare.
- CollaborationResult: The result of a collaboration, usually the result will be with a enum type.
- Behavior: It'a union of all collaboration results.

## Current Behaviors defined

```rust
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
```

## Reputation Score

Each Behavior(CollaborationResult) has a corresponding reputation score.

## Interfaces

```rust
fn add_behavior_score_by_behavior(target: &AccountId, behavior: &Behavior);
```

This interface will be called in collaboration modules(bounties). The `behavior` param will be different based on the caller module's business.
