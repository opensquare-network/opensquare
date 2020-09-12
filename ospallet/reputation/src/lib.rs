#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_event, decl_module, decl_storage};
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
                Self::deposit_event(RawEvent::ReputationAdded(target.clone(), new_score));
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

#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_io::TestExternalities;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    pub fn new_test_ext() -> TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        TestExternalities::new(t)
    }

    impl_outer_origin! {
        pub enum Origin for Test where system = frame_system {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }
    impl frame_system::Trait for Test {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type AvailableBlockRatio = AvailableBlockRatio;
        type MaximumBlockLength = MaximumBlockLength;
        type Version = ();
        type ModuleToIndex = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    impl Trait for Test {
        type Event = ();
    }

    type Reputation = Module<Test>;

    #[test]
    fn add_behavior_score_by_behavior_works() {
        new_test_ext().execute_with(|| {
            let account = 1;
            Reputation::add_behavior_score_by_behavior(
                &account,
                &Behavior::BountyResolve(BountyResolveCollaborationResult::Success),
            );

            let score = Reputation::behavior_score(&account);
            assert_eq!(10, score);
        });
    }

    #[test]
    fn get_behavior_score_works() {
        let score = Reputation::get_behavior_score(&Behavior::BountyResolve(
            BountyResolveCollaborationResult::Fail,
        ));
        assert_eq!(-2, score);
    }
}
