use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// Substrate
use sp_runtime::RuntimeDebug;

use opensquare_primitives::SdDigest;

use crate::{BalanceOf, CurrencyIdOf};

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BountyCategory {
    Development,
    Design,
    Document,
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum BountyState {
    Applying,
    // Accepted by the council
    Accepted,
    // Rejected by the council
    Rejected,
    // Closed by funder
    Closed,
    // Closed by the council
    CouncilClosed,
    Assigned,
    Outdated,
    Submitted,
    Resolved,
}

impl Default for BountyState {
    fn default() -> Self {
        BountyState::Applying
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Bounty<AccountId, CurrencyId, Balance> {
    V1(BountyMetaData<AccountId, CurrencyId, Balance>),
}

pub type BountyOf<T> = Bounty<<T as frame_system::Trait>::AccountId, CurrencyIdOf<T>, BalanceOf<T>>;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BountyMetaData<AccountId, CurrencyId, Balance> {
    pub owner: AccountId,
    pub currency_id: CurrencyId,
    pub payment: Balance,
    pub digest: SdDigest,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SettingData {
    pub category: BountyCategory,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CloseReason {
    Outdated,
    InvalidState,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum HunterBountyState {
    Hunting,
    Processing,
}

impl Default for HunterBountyState {
    fn default() -> Self {
        Self::Hunting
    }
}
