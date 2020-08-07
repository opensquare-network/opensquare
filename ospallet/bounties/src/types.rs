use codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

// Substrate
use sp_runtime::RuntimeDebug;

use opensquare_primitives::SdDigest;

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
    Accepted,
    Rejected,
    Idle,
    Assigned,
    Outdated,
    Reviewing,
    Resolved,
}

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
