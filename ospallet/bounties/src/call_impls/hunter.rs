use frame_support::{
    dispatch::DispatchResult, ensure, storage::StorageDoubleMap, IterableStorageDoubleMap,
};

use opensquare_primitives::BountyId;
use ospallet_reputation::{
    Behavior, BountyRemarkCollaborationResult, BountyResolveCollaborationResult, ReputationBuilder,
};

use crate::types::{BountyState, HunterBountyState};
use crate::{Error, HunterBounties, HuntingForBounty, Module, RawEvent, Trait};

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
        HuntingForBounty::<T>::insert(bounty_id, &hunter, true);

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

    pub fn cancel_bounty_hunting_impl(bounty_id: BountyId, hunter: T::AccountId) -> DispatchResult {
        ensure!(
            Self::hunting_for_bounty(&bounty_id, &hunter),
            Error::<T>::NotHunter
        );

        HuntingForBounty::<T>::remove(&bounty_id, &hunter);
        HunterBounties::<T>::remove(&hunter, &bounty_id);
        Self::deposit_event(RawEvent::CancelHuntBounty(bounty_id, hunter));
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

        T::ReputationBuilder::add_behavior_score_by_behavior(
            &hunter,
            &Behavior::BountyResolve(BountyResolveCollaborationResult::Fail),
        );

        Self::deposit_event(RawEvent::Resign(bounty_id, hunter));

        Self::change_state(bounty_id, BountyState::Accepted);
        Self::deposit_event(RawEvent::Accept(bounty_id));

        Ok(())
    }

    pub fn remark_bounty_funder_impl(
        bounty_id: BountyId,
        hunter: T::AccountId,
        _remark: BountyRemarkCollaborationResult,
    ) -> DispatchResult {
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Resolved,
            Error::<T>::InvalidState
        );
        ensure!(
            Self::hunted_for_bounty(&bounty_id) == hunter,
            Error::<T>::NotHunter
        );

        // remove hunter
        Self::remove_hunters_for_bounty(bounty_id);
        Self::deposit_event(RawEvent::HunterRemark(bounty_id, _remark));

        let bounty = Self::get_bounty(&bounty_id)?;
        let funder = Self::get_funder(&bounty);

        T::ReputationBuilder::add_behavior_score_by_behavior(
            &funder,
            &Behavior::BountyRemark(_remark),
        );

        Ok(())
    }
}
