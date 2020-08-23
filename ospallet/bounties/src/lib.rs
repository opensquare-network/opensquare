#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::EnsureOrigin,
    IterableStorageDoubleMap,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{BlakeTwo256, Hash, SaturatedConversion, StaticLookup},
    Percent,
};
use sp_std::{marker::PhantomData, prelude::*, result};

// orml
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use orml_utilities::with_transaction_result;

use opensquare_primitives::BountyId;

use crate::types::{Bounty, BountyOf, BountyRemark, BountyState, CloseReason, HunterBountyState};
use frame_support::traits::{BalanceStatus, Get};

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

    type CouncilAccount: Get<Self::AccountId>;

    type CouncilFee: Get<Percent>;

    type DetermineBountyId: BountyIdFor<Self::AccountId>;
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

    }
}
decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>
    {
        CreateBounty(AccountId, BountyId),
        Apply(BountyId),
        Approve(BountyId),
        Accept(BountyId),
        Reject(BountyId),
        Close(BountyId, Balance),
        HuntBounty(BountyId, AccountId),
        AssignBounty(BountyId, AccountId),
        OutdateBounty(BountyId),
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
            double_map hasher(identity) BountyId, hasher(blake2_128_concat) T::AccountId => Option<()>;
        /// record a hunted bounty has been doing by who(single hunter)
        pub HuntedForBounty get(fn hunted_for_bounty): map hasher(identity) BountyId => T::AccountId;

        /// record bounties for a hunter, include hunting and hunted(in processing)
        pub HunterBounties get(fn hunter_bounties):
            double_map hasher(blake2_128_concat) T::AccountId, hasher(identity) BountyId => HunterBountyState;

        pub MaxHoldingBounties get(fn max_holding_bounties): u32 = 10;
        pub OutdatedHeight get(fn outdated_height): T::BlockNumber = 1000.saturated_into();
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
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

        /// apply a bounty after creating a bounty.
        #[weight = 0]
        fn apply_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::apply_bounty_impl(who, bounty_id)?;
            Ok(())
        }

        #[weight = 0]
        fn assign_bounty(origin, bounty_id: BountyId, assign_to: <T::Lookup as StaticLookup>::Source) -> DispatchResult {
            let funder = ensure_signed(origin)?;
            let assign_to = T::Lookup::lookup(assign_to)?;

            Self::assign_bounty_impl(bounty_id, funder, assign_to)
        }

        #[weight = 0]
        fn resolve_bounty_and_remark(origin, bounty_id: BountyId, remark: BountyRemark) -> DispatchResult {
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
        fn cancel_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::cancel_bounty_impl(bounty_id, hunter)
        }

        #[weight = 0]
        fn resign_from_bounty(origin, bounty_id: BountyId) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::resign_from_bounty_impl(bounty_id, hunter)
        }

        #[weight = 0]
        fn remark_bounty_funder(origin, bounty_id: BountyId, remark: BountyRemark) -> DispatchResult {
            let hunter = ensure_signed(origin)?;
            Self::remark_bounty_funder_impl(bounty_id, hunter, remark)
        }

    }
}

impl<T: Trait> Module<T> {
    fn get_bounty(id: &BountyId) -> result::Result<BountyOf<T>, DispatchError> {
        let b = Self::bounties(id).ok_or(Error::<T>::NotExisted)?;
        Ok(b)
    }

    fn get_funder(bounty: &BountyOf<T>) -> T::AccountId {
        match bounty {
            Bounty::V1(ref metadata) => metadata.owner.clone(),
        }
    }

    fn check_funder(caller: &T::AccountId, bounty: &BountyOf<T>) -> DispatchResult {
        let funder = Self::get_funder(bounty);
        ensure!(&funder == caller, Error::<T>::NotFunder);
        Ok(())
    }

    fn parse_payment(bounty: &BountyOf<T>) -> (CurrencyIdOf<T>, BalanceOf<T>) {
        match bounty {
            Bounty::V1(ref metadata) => (metadata.currency_id, metadata.payment),
        }
    }

    fn remove_hunters_for_bounty(bounty_id: BountyId) {
        // remove hunters for a bounty
        let mut hunters = HuntingForBounty::<T>::drain_prefix(bounty_id)
            .map(|(a, _)| a)
            .collect::<Vec<_>>();
        let hunted_hunter = HuntedForBounty::<T>::take(&bounty_id);
        // hunters.extend(accounts);
        hunters.push(hunted_hunter);
        // remove bounty for hunters
        for hunter in hunters {
            HunterBounties::<T>::remove(hunter, bounty_id)
        }
    }

    fn remove_hunter_for_bounty(bounty_id: BountyId) {
        // 1
        let hunter = HuntedForBounty::<T>::take(bounty_id);
        // 2
        HunterBounties::<T>::remove(&hunter, bounty_id);
        // 3
        HuntingForBounty::<T>::remove(bounty_id, &hunter);
    }

    fn change_state(bounty_id: BountyId, state: BountyState) {
        match state {
            BountyState::Assigned => {
                AssignedHeight::<T>::insert(bounty_id, frame_system::Module::<T>::block_number());
            }
            BountyState::Applying => {
                ApprovedHeight::<T>::insert(bounty_id, frame_system::Module::<T>::block_number());
            }
            _ => { /* do nothing*/ }
        }
        BountyStateOf::insert(bounty_id, state);
    }
}

// funder call
impl<T: Trait> Module<T> {
    fn create_bounty_impl(creator: T::AccountId, bounty: BountyOf<T>) -> DispatchResult {
        let bounty_id = T::DetermineBountyId::bounty_id_for(&creator);

        ensure!(!BountyStateOf::contains_key(bounty_id), Error::<T>::Existed);
        ensure!(Self::bounties(bounty_id).is_none(), Error::<T>::Existed);

        Self::check_funder(&creator, &bounty)?;

        // reserve balance and other init
        Self::handle_init_bounty(&creator, &bounty)?;

        Bounties::<T>::insert(bounty_id, bounty);
        BountiesOf::<T>::mutate(&creator, |list| {
            if !list.contains(&bounty_id) {
                list.push(bounty_id);
            }
        });
        Self::change_state(bounty_id, BountyState::Creating);
        Self::deposit_event(RawEvent::CreateBounty(creator, bounty_id));
        Ok(())
    }

    fn handle_init_bounty(funder: &T::AccountId, bounty: &BountyOf<T>) -> DispatchResult {
        let (id, locked) = Self::parse_payment(&bounty);

        if !T::Currency::can_reserve(id, funder, locked) {
            Err(Error::<T>::CantPay)?
        }

        T::Currency::reserve(id, funder, locked)?;

        Ok(())
    }

    fn apply_bounty_impl(caller: T::AccountId, bounty_id: BountyId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Creating,
            Error::<T>::InvalidState
        );
        let b = Self::get_bounty(&bounty_id)?;
        Self::check_funder(&caller, &b)?;
        // todo do check

        Self::change_state(bounty_id, BountyState::Applying);
        Self::deposit_event(RawEvent::Apply(bounty_id));
        Ok(())
    }

    fn close_bounty_impl(funder: T::AccountId, bounty_id: BountyId) -> DispatchResult {
        // No meaning to close a rejected bounty
        let state = Self::bounty_state_of(bounty_id);
        ensure!(
            (state != BountyState::Rejected)
                || (state != BountyState::Closed)
                || (state != BountyState::Outdated)
                || (state != BountyState::Resolved),
            Error::<T>::InvalidState
        );

        let bounty = Self::get_bounty(&bounty_id)?;
        let (id, locked) = Self::parse_payment(&bounty);

        Self::check_funder(&funder, &bounty)?;

        // release reserved balance
        let remaining = T::Currency::unreserve(id, &funder, locked);
        // remove hunter for a bounty
        Self::remove_hunters_for_bounty(bounty_id);

        Self::change_state(bounty_id, BountyState::Closed);
        Self::deposit_event(RawEvent::Close(bounty_id, remaining));
        Ok(())
    }

    fn assign_bounty_impl(
        bounty_id: BountyId,
        funder: T::AccountId,
        hunter: T::AccountId,
    ) -> DispatchResult {
        let state = Self::bounty_state_of(bounty_id);
        ensure!(
            (state == BountyState::Accepted) || (state == BountyState::Assigned), // could be assigned again
            Error::<T>::InvalidState
        );
        let bounty = Self::get_bounty(&bounty_id)?;
        Self::check_funder(&funder, &bounty)?;

        // judge new hunter is in hunting list
        ensure!(
            Self::hunting_for_bounty(bounty_id, &hunter).is_some(),
            Error::<T>::NotHunter
        );
        HuntedForBounty::<T>::try_mutate_exists(bounty_id, |option| -> DispatchResult {
            if let Some(old_hunter) = option {
                if old_hunter == &hunter {
                    Err(Error::<T>::AlreadyAssigned)?
                }

                // change old hunter state, if old not exist, do nothing
                HunterBounties::<T>::insert(old_hunter, bounty_id, HunterBountyState::Hunting);
            }
            // set new hunter state
            HunterBounties::<T>::insert(&hunter, bounty_id, HunterBountyState::Processing);
            // replace old to new
            *option = Some(hunter.clone());
            Ok(())
        })?;

        Self::change_state(bounty_id, BountyState::Assigned);
        Self::deposit_event(RawEvent::AssignBounty(bounty_id, hunter));
        Ok(())
    }

    // todo, need remark score
    fn resolve_bounty_and_remark_impl(
        bounty_id: BountyId,
        funder: T::AccountId,
        remark: BountyRemark,
    ) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Submitted,
            Error::<T>::InvalidState
        );
        let bounty = Self::get_bounty(&bounty_id)?;
        Self::check_funder(&funder, &bounty)?;

        // TODO maybe other check

        // release currency
        let hunter = Self::hunted_for_bounty(&bounty_id);
        let (id, locked) = Self::parse_payment(&bounty);

        // todo, can't impl yet
        // let fee =T::CouncilFee::get() *  locked ;
        // let council_account = T::CouncilAccount::get();
        // // todo may be use log to print remaining
        // let _ = T::Currency::repatriate_reserved(
        //     id,
        //     &funder,
        //     &hunter,
        //     locked - fee,
        //     BalanceStatus::Free,
        // )?;
        // let _ = T::Currency::repatriate_reserved(
        //     id,
        //     &council_account,
        //     &hunter,
        //     fee,
        //     BalanceStatus::Free,
        // )?;

        // remove hunter
        Self::remove_hunters_for_bounty(bounty_id);

        Self::change_state(bounty_id, BountyState::Resolved);
        Self::deposit_event(RawEvent::Resolve(bounty_id));
        // TODO maybe delete storage to save disk space

        Ok(())
    }
}

// council call
impl<T: Trait> Module<T> {
    fn examine_bounty_impl(bounty_id: BountyId, accepted: bool) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Applying,
            Error::<T>::InvalidState
        );
        if accepted {
            Self::change_state(bounty_id, BountyState::Accepted);
            Self::deposit_event(RawEvent::Accept(bounty_id));
        } else {
            Self::change_state(bounty_id, BountyState::Rejected);
            Self::deposit_event(RawEvent::Reject(bounty_id));
        }
        Ok(())
    }
    fn force_close_bounty_impl(bounty_id: BountyId, reason: CloseReason) -> DispatchResult {
        let bounty = Self::get_bounty(&bounty_id)?;
        let funder = Self::get_funder(&bounty);
        let (id, locked) = Self::parse_payment(&bounty);
        let state = Self::bounty_state_of(bounty_id);

        with_transaction_result(|| {
            // remove hunter for a bounty
            Self::remove_hunters_for_bounty(bounty_id);
            match reason {
                CloseReason::Outdated => {
                    ensure!(
                        (state == BountyState::Accepted) || (state == BountyState::Assigned),
                        Error::<T>::InvalidState
                    );
                    let height = Self::assigned_height(&bounty_id);
                    let current_height = frame_system::Module::<T>::block_number();
                    if height + Self::outdated_height() > current_height {
                        Err(Error::<T>::ValidBounty)?;
                    }
                    // release reserved balance, todo maybe use log to print it
                    let _remaining = T::Currency::unreserve(id, &funder, locked);

                    Self::change_state(bounty_id, BountyState::Outdated);
                    Self::deposit_event(RawEvent::OutdateBounty(bounty_id));
                }
                CloseReason::Resolved => {
                    ensure!(
                        Self::bounty_state_of(bounty_id) == BountyState::Assigned,
                        Error::<T>::InvalidState
                    );
                    // TODO how to resolve?
                } // TODO other reason
            }
            Ok(())
        })
    }
}

// hunter call
impl<T: Trait> Module<T> {
    fn hunt_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Accepted,
            Error::<T>::InvalidState
        );

        // this count include hunting and processing, if not need processing, should filter this
        let count = HunterBounties::<T>::iter_prefix(&hunter).count();
        ensure!(
            count as u32 <= Self::max_holding_bounties(),
            Error::<T>::TooManyHuntedBounties
        );
        ensure!(
            !HunterBounties::<T>::contains_key(&hunter, &bounty_id),
            Error::<T>::AlreadyHunted
        );

        HunterBounties::<T>::insert(&hunter, bounty_id, HunterBountyState::Hunting);
        HuntingForBounty::<T>::insert(bounty_id, &hunter, ());

        Self::deposit_event(RawEvent::HuntBounty(bounty_id, hunter));
        Ok(())
    }

    fn submit_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Assigned,
            Error::<T>::InvalidState
        );

        ensure!(
            Self::hunted_for_bounty(&bounty_id) == hunter,
            Error::<T>::NotHunter
        );

        Self::change_state(bounty_id, BountyState::Submitted);
        Self::deposit_event(RawEvent::Submit(bounty_id));
        Ok(())
    }

    fn cancel_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Applying,
            Error::<T>::InvalidState
        );
        ensure!(
            Self::hunting_for_bounty(&bounty_id, &hunter).is_some(),
            Error::<T>::NotHunter
        );

        HuntingForBounty::<T>::remove(&bounty_id, &hunter);
        HunterBounties::<T>::remove(&hunter, &bounty_id);
        Ok(())
    }

    fn resign_from_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Assigned,
            Error::<T>::InvalidState
        );
        ensure!(
            Self::hunted_for_bounty(&bounty_id) == hunter,
            Error::<T>::NotHunter
        );

        Self::remove_hunter_for_bounty(bounty_id);

        Self::deposit_event(RawEvent::Resign(bounty_id, hunter));

        Self::change_state(bounty_id, BountyState::Accepted);
        Self::deposit_event(RawEvent::Apply(bounty_id));

        Ok(())
    }

    fn remark_bounty_funder_impl(
        bounty_id: BountyId,
        hunter: T::AccountId,
        remark: BountyRemark,
    ) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Resolved,
            Error::<T>::InvalidState
        );
        ensure!(
            Self::hunted_for_bounty(&bounty_id) == hunter,
            Error::<T>::NotHunter
        );

        Ok(())
    }
}
