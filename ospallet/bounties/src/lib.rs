#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::EnsureOrigin,
};
use frame_system::ensure_signed;
use sp_runtime::traits::{BlakeTwo256, Hash, SaturatedConversion, StaticLookup};
use sp_std::{marker::PhantomData, prelude::*, result};

// orml
use orml_traits::{MultiCurrency, MultiReservableCurrency};

use opensquare_primitives::BountyId;

use crate::types::{Bounty, BountyOf, BountyState};

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
            .saturated_into() as u32;
        let mut buf = Vec::new();
        buf.extend_from_slice(origin.as_ref());
        buf.extend_from_slice(&nonce.to_le_bytes());
        BlakeTwo256::hash(&buf)
    }
}

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: MultiCurrency<Self::AccountId> + MultiReservableCurrency<Self::AccountId>;

    type CouncilOrigin: EnsureOrigin<Self::Origin>;

    type DetermineBountyId: BountyIdFor<Self::AccountId>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        NotExisted,
        Existed,
        InvalidBounty,
        InvalidState,
        NotFunder,
        /// beyond limit of max hunted bounties
        TooManyHuntedBounties,
        /// this hunter already hunt this bounty
        AlreadyHunted,
        /// not hunter for this bounty
        NotHunter,
    }
}
decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId
    {
        CreateBounty(AccountId, BountyId),
        Apply(BountyId),
        Approve(BountyId),
        Accept(BountyId),
        Reject(BountyId),
        HuntBounty(BountyId, AccountId),
        AssignBounty(BountyId, AccountId),
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

        pub HuntersFor get(fn hunters_for):
            double_map hasher(identity) BountyId, hasher(blake2_128_concat) T::AccountId => Option<()>;
        pub HuntedBountiesFor get(fn hunted_bounties_for): map hasher(blake2_128_concat)
            T::AccountId => Vec<BountyId>;

        // todo, may use struct instead of tuple
        pub HuntedBounties get(fn hunted_bounties): map hasher(identity) BountyId => Option<(T::AccountId, T::BlockNumber)>;

        pub MaxHoldingBounties get(fn max_holding_bounties): u32 = 10;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = 0]
        fn create_bounty(origin, bounty: Bounty<T::AccountId, CurrencyIdOf<T>, BalanceOf<T>>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::create_bounty_impl(who, bounty)?;
            Ok(())
        }

        #[weight = 0]
        fn apply_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::apply_bounty_impl(who, bounty_id)?;
            Ok(())
        }

        #[weight = 0]
        fn review_bounty(origin, bounty_id: BountyId, accepted: bool) -> DispatchResult {
            T::CouncilOrigin::ensure_origin(origin)?;
            Self::review_bounty_impl(bounty_id, accepted)?;
            Ok(())
        }

        #[weight = 0]
        fn hunt_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::hunt_bounty_impl(bounty_id, who)
        }

        #[weight = 0]
        fn assign_bounty(origin, bounty_id: BountyId, assign_to: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let funder = ensure_signed(origin)?;
            let assign_to = T::Lookup::lookup(assign_to)?;
            Self::assign_bounty_impl(bounty_id, funder, assign_to)
        }
    }
}

impl<T: Trait> Module<T> {
    fn get_bounty(id: &BountyId) -> result::Result<BountyOf<T>, DispatchError> {
        let b = Self::bounties(id).ok_or(Error::<T>::NotExisted)?;
        Ok(b)
    }
    fn check_caller(caller: &T::AccountId, bounty: &BountyOf<T>) -> DispatchResult {
        match bounty {
            Bounty::V1(ref metadata) => {
                if metadata.owner != *caller {
                    Err(Error::<T>::NotFunder)?
                }
            }
        }
        Ok(())
    }
    fn create_bounty_impl(creator: T::AccountId, bounty: BountyOf<T>) -> DispatchResult {
        let bounty_id = T::DetermineBountyId::bounty_id_for(&creator);
        ensure!(Self::bounties(bounty_id).is_none(), Error::<T>::Existed);

        Self::check_caller(&creator, &bounty)?;
        BountyStateOf::insert(bounty_id, BountyState::Creating);
        Bounties::<T>::insert(bounty_id, bounty);
        BountiesOf::<T>::mutate(&creator, |list| {
            if !list.contains(&bounty_id) {
                list.push(bounty_id);
            }
        });
        Self::deposit_event(RawEvent::CreateBounty(creator, bounty_id));
        Ok(())
    }
    fn apply_bounty_impl(caller: T::AccountId, bounty_id: BountyId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Creating,
            Error::<T>::InvalidState
        );
        let b = Self::get_bounty(&bounty_id)?;
        Self::check_caller(&caller, &b)?;
        // todo do check

        BountyStateOf::insert(bounty_id, BountyState::Applying);
        Self::deposit_event(RawEvent::Apply(bounty_id));
        Ok(())
    }
    fn review_bounty_impl(bounty_id: BountyId, accepted: bool) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Applying,
            Error::<T>::InvalidState
        );
        if accepted {
            BountyStateOf::insert(bounty_id, BountyState::Accepted);
            Self::deposit_event(RawEvent::Accept(bounty_id));
        } else {
            BountyStateOf::insert(bounty_id, BountyState::Rejected);
            Self::deposit_event(RawEvent::Reject(bounty_id));
        }
        Ok(())
    }
    fn hunt_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Accepted,
            Error::<T>::InvalidState
        );
        ensure!(
            Self::hunted_bounties_for(&hunter).len() as u32 <= Self::max_holding_bounties(),
            Error::<T>::TooManyHuntedBounties
        );

        ensure!(Self::hunters_for(bounty_id, &hunter).is_none(), Error::<T>::AlreadyHunted);
        HuntersFor::<T>::insert(bounty_id, &hunter, ());

        Self::deposit_event(RawEvent::HuntBounty(bounty_id, hunter));
        Ok(())
    }
    fn assign_bounty_impl(bounty_id: BountyId, funder: T::AccountId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Accepted,
            Error::<T>::InvalidState
        );
        let bounty = Self::get_bounty(&bounty_id)?;
        Self::check_caller(&funder, &bounty)?;
        ensure!(Self::hunters_for(bounty_id, &hunter).is_some(), Error::<T>::NotHunter);


        HuntedBounties::<T>::insert(bounty_id, (hunter.clone(), frame_system::Module::<T>::block_number()));

        BountyStateOf::insert(bounty_id, BountyState::Assigned);
        Self::deposit_event(RawEvent::AssignBounty(bounty_id, hunter));
        Ok(())
    }
}
