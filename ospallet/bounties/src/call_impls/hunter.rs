use frame_support::{
    dispatch::DispatchResult,
    ensure,
    IterableStorageDoubleMap,
    storage::StorageDoubleMap,
};

use opensquare_primitives::BountyId;

use crate::{
    Error, HunterBounties, HuntingForBounty, Module,
    RawEvent,
    Trait,
};
use crate::types::{BountyRemark, BountyState, HunterBountyState};

impl<T: Trait> Module<T> {
    pub fn hunt_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        let state = Self::bounty_state_of(bounty_id);
        ensure!(
            (state == BountyState::Accepted)
            || (state == BountyState::Assigned)
            || (state == BountyState::Submitted),
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

    pub fn submit_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Assigned,
            Error::<T>::InvalidState
        );

        ensure!(
            Self::hunted_for_bounty(&bounty_id) == hunter,
            Error::<T>::NotAssignee
        );

        Self::change_state(bounty_id, BountyState::Submitted);
        Self::deposit_event(RawEvent::Submit(bounty_id));
        Ok(())
    }

    pub fn cancel_bounty_hunting_imple(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
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

    pub fn resign_from_bounty_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
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
        Self::deposit_event(RawEvent::Accept(bounty_id));

        Ok(())
    }

    pub fn remark_bounty_funder_impl(
        bounty_id: BountyId,
        hunter: T::AccountId,
        _remark: BountyRemark,
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
