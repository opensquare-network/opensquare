#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    traits::EnsureOrigin,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{BlakeTwo256, Hash, SaturatedConversion, StaticLookup},
    Percent,
};
use sp_std::{marker::PhantomData, prelude::*};

use opensquare_primitives::BountyId;
// orml
use orml_traits::{MultiCurrency, MultiReservableCurrency};

use crate::types::{Bounty, BountyOf, BountyState, CloseReason, HunterBountyState};

use ospallet_mining::MiningPowerBuilder;
use ospallet_reputation::{BountyRemarkCollaborationResult, ReputationBuilder};

mod call_impls;
mod types;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
pub type CurrencyIdOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId;

/// A function that generates an `AccountId` for a contract upon instantiation.
pub trait BountyIdFor<AccountId> {
    fn bounty_id_for(origin: &AccountId) -> BountyId;
}

/// Simple BountyId determiner.
///
/// Address calculated from the code (of the constructor), input data to the constructor,
/// and the account id that requested the account creation.
///
/// Formula: `blake2_256(blake2_256(code) + blake2_256(data) + origin)`
pub struct SimpleBountyIdDeterminer<T: Trait>(PhantomData<T>);

impl<T: Trait> BountyIdFor<T::AccountId> for SimpleBountyIdDeterminer<T>
where
    T::AccountId: AsRef<[u8]>,
{
    fn bounty_id_for(origin: &T::AccountId) -> BountyId {
        let nonce: u32 = frame_system::Module::<T>::account(origin)
            .nonce
            .saturated_into::<u32>();
        let mut buf = Vec::new();
        buf.extend_from_slice(origin.as_ref());
        buf.extend_from_slice(&nonce.to_le_bytes());
        BlakeTwo256::hash(&buf)
    }
}

pub trait BountyResolved<T: Trait> {
    fn after_bounty_resolved(_bounty: &BountyOf<T>) {}
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl<T: Trait> BountyResolved<T> for Tuple {
    fn after_bounty_resolved(_bounty: &BountyOf<T>) {
        for_tuples!( #( Tuple::after_bounty_resolved(_bounty); )* );
    }
}

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: MultiCurrency<Self::AccountId> + MultiReservableCurrency<Self::AccountId>;

    type CouncilOrigin: EnsureOrigin<Self::Origin>;

    type CouncilAccount: Get<Self::AccountId>;

    type CouncilFee: Get<Percent>;

    type DetermineBountyId: BountyIdFor<Self::AccountId>;

    type BountyResolved: BountyResolved<Self>;

    type ReputationBuilder: ReputationBuilder<Self::AccountId>;

    type MiningPowerBuilder: MiningPowerBuilder<Self::AccountId>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        NotExisted,
        Existed,
        InvalidBounty,
        CantPay,
        ValidBounty,
        InvalidState,
        NotFunder,
        /// beyond limit of max hunted bounties
        TooManyHuntedBounties,
        /// this hunter already hunt this bounty
        AlreadyHunted,
        /// this bounty already assgined to this hunter
        AlreadyAssigned,
        /// not hunter for this bounty
        NotHunter,
        /// not assignee of this bounty
        NotAssignee,
    }
}
decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>
    {
        ApplyBounty(AccountId, BountyId),
        Accept(BountyId),
        Reject(BountyId),
        Close(BountyId, Balance),
        ForceClosed(BountyId, CloseReason, Balance),
        HuntBounty(BountyId, AccountId),
        CancelHuntBounty(BountyId, AccountId),
        AssignBounty(BountyId, AccountId),
        Submit(BountyId),
        Resign(BountyId, AccountId),
        Resolve(BountyId),
    }
);
decl_storage! {
    trait Store for Module<T: Trait> as OSBounties {
        /// Bounties basic info of a bounty_id
        pub Bounties get(fn bounties): map hasher(identity)
            BountyId => Option<Bounty<T::AccountId, CurrencyIdOf<T>, BalanceOf<T>>>;
        /// Record bounties of an accountid
        pub BountiesOf get(fn bounties_of): map hasher(blake2_128_concat)
            T::AccountId => Vec<BountyId>;
        /// Bounty state of a bounty_id
        pub BountyStateOf get(fn bounty_state_of): map hasher(identity) BountyId => BountyState;

        pub ApprovedHeight get(fn approved_height): map hasher(identity) BountyId => T::BlockNumber;
        pub AssignedHeight get(fn assigned_height): map hasher(identity) BountyId => T::BlockNumber;

        /// mark this bounty has been hunting by who
        pub HuntingForBounty get(fn hunting_for_bounty):
            double_map hasher(identity) BountyId, hasher(blake2_128_concat) T::AccountId => bool;
        /// record a hunted bounty has been doing by who(single hunter)
        HuntedForBounty get(fn hunted_for_bounty): map hasher(identity) BountyId => T::AccountId;

        /// record bounties for a hunter, include hunting and hunted(in processing)
        pub HunterBounties get(fn hunter_bounties):
            double_map hasher(blake2_128_concat) T::AccountId, hasher(identity) BountyId => Option<HunterBountyState>;

        pub MaxHoldingBounties get(fn max_holding_bounties): u32 = 10;
        pub OutdatedHeight get(fn outdated_height): T::BlockNumber = 1000.saturated_into();

        pub CurrencyRatios get(fn currency_ratios) config(): map hasher(blake2_128_concat) CurrencyIdOf<T> => u128;
    }
        add_extra_genesis {
            config(dummy): u32;
            build(|config| {
                for (currency_id, power) in &config.currency_ratios {
                    CurrencyRatios::<T>::insert(currency_id, power);
                }
            })
        }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
        // funder call
        #[weight = 0]
        fn create_bounty(origin, bounty: Bounty<T::AccountId, CurrencyIdOf<T>, BalanceOf<T>>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::create_bounty_impl(who, bounty)?;
            Ok(())
        }

        #[weight = 0]
        fn close_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::close_bounty_impl(who, bounty_id)?;
            Ok(())
        }

        #[weight = 0]
        fn assign_bounty(origin, bounty_id: BountyId, assign_to: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let funder = ensure_signed(origin)?;
            let assign_to = T::Lookup::lookup(assign_to)?;

            Self::assign_bounty_impl(bounty_id, funder, assign_to)
        }

        #[weight = 0]
        fn resolve_bounty_and_remark(origin, bounty_id: BountyId, remark: BountyRemarkCollaborationResult) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::resolve_bounty_and_remark_impl(bounty_id, who, remark)?;
            Ok(())
        }

        // council call
        /// council accept or reject a bounty
        #[weight = 0]
        fn examine_bounty(origin, bounty_id: BountyId, accepted: bool) -> DispatchResult {
            T::CouncilOrigin::ensure_origin(origin)?;
            Self::examine_bounty_impl(bounty_id, accepted)?;
            Ok(())
        }

        #[weight = 0]
        fn force_close_bounty(origin, bounty_id: BountyId, reason: CloseReason) -> DispatchResult {
            T::CouncilOrigin::ensure_origin(origin)?;
            Self::force_close_bounty_impl(bounty_id, reason)
        }

        // hunter call
        #[weight = 0]
        fn hunt_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::hunt_bounty_impl(bounty_id, who)
        }

        #[weight = 0]
        fn submit_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::submit_bounty_impl(bounty_id, hunter)
        }

        #[weight = 0]
        fn cancel_hunt_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::cancel_bounty_hunting_impl(bounty_id, hunter)
        }

        #[weight = 0]
        fn resign_from_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::resign_from_bounty_impl(bounty_id, hunter)
        }

        #[weight = 0]
        fn remark_bounty_funder(origin, bounty_id: BountyId, remark: BountyRemarkCollaborationResult) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::remark_bounty_funder_impl(bounty_id, hunter, remark)
        }

    }
}
