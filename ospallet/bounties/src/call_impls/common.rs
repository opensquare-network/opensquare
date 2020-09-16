use frame_support::{
    dispatch::DispatchError,
    dispatch::DispatchResult,
    ensure,
    storage::{IterableStorageDoubleMap, StorageDoubleMap, StorageMap},
};
use sp_std::{prelude::*, result};

use opensquare_primitives::BountyId;

use crate::types::{Bounty, BountyOf, BountyState};
use crate::{
    ApprovedHeight, AssignedHeight, BalanceOf, BountyStateOf, CurrencyIdOf, Error, HuntedForBounty,
    HunterBounties, HuntingForBounty, Module, Trait,
};

impl<T: Trait> Module<T> {
    pub fn get_bounty(id: &BountyId) -> result::Result<BountyOf<T>, DispatchError> {
        let b = Self::bounties(id).ok_or(Error::<T>::NotExisted)?;
        Ok(b)
    }

    pub fn get_funder(bounty: &BountyOf<T>) -> T::AccountId {
        match bounty {
            Bounty::V1(ref metadata) => metadata.owner.clone(),
        }
    }

    pub fn get_currency_id(bounty: &BountyOf<T>) -> CurrencyIdOf<T> {
        match bounty {
            Bounty::V1(ref metadata) => metadata.currency_id.clone(),
        }
    }

    pub fn check_funder(caller: &T::AccountId, bounty: &BountyOf<T>) -> DispatchResult {
        let funder = Self::get_funder(bounty);
        ensure!(&funder == caller, Error::<T>::NotFunder);
        Ok(())
    }

    pub fn parse_payment(bounty: &BountyOf<T>) -> (CurrencyIdOf<T>, BalanceOf<T>) {
        match bounty {
            Bounty::V1(ref metadata) => (metadata.currency_id, metadata.payment),
        }
    }

    pub fn remove_hunters_for_bounty(bounty_id: BountyId) {
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

    pub fn remove_hunter_for_bounty(bounty_id: BountyId) {
        // 1
        let hunter = HuntedForBounty::<T>::take(bounty_id);
        // 2
        HunterBounties::<T>::remove(&hunter, bounty_id);
        // 3
        HuntingForBounty::<T>::remove(bounty_id, &hunter);
    }

    pub fn change_state(bounty_id: BountyId, state: BountyState) {
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

    pub fn check_bounty_can_be_closed(bounty_id: BountyId) -> DispatchResult {
        let state = Self::bounty_state_of(bounty_id);
        ensure!(
            // No meaning to close a rejected bounty
            (state != BountyState::Rejected)
                || (state != BountyState::Closed)
                || (state != BountyState::Outdated)
                || (state != BountyState::Resolved),
            Error::<T>::InvalidState
        );

        Ok(())
    }
}
