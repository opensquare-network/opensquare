use frame_support::{
    dispatch::DispatchResult,
    ensure,
    storage::{StorageDoubleMap, StorageMap},
    traits::{BalanceStatus, Get},
};

use opensquare_primitives::BountyId;
use orml_traits::MultiReservableCurrency;
use ospallet_mining::MiningPowerBuilder;
use ospallet_reputation::{
    Behavior, BountyRemarkCollaborationResult, BountyResolveCollaborationResult, ReputationBuilder,
};
use sp_runtime::traits::SaturatedConversion;

use crate::types::{BountyOf, BountyState, HunterBountyState};
use crate::{
    BalanceOf, Bounties, BountiesOf, BountyIdFor, BountyResolved, BountyStateOf, Error,
    HuntedForBounty, HunterBounties, Module, RawEvent, Trait,
};

impl<T: Trait> Module<T> {
    pub fn create_bounty_impl(creator: T::AccountId, bounty: BountyOf<T>) -> DispatchResult {
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
        Self::change_state(bounty_id, BountyState::Applying);
        Self::deposit_event(RawEvent::ApplyBounty(creator, bounty_id));
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

    pub fn close_bounty_impl(funder: T::AccountId, bounty_id: BountyId) -> DispatchResult {
        let bounty = Self::get_bounty(&bounty_id)?;
        Self::check_funder(&funder, &bounty)?;
        Self::check_bounty_can_be_closed(bounty_id)?;

        let (id, locked) = Self::parse_payment(&bounty);
        // release reserved balance
        let remaining = T::Currency::unreserve(id, &funder, locked);
        // remove hunter for a bounty
        Self::remove_hunters_for_bounty(bounty_id);

        Self::change_state(bounty_id, BountyState::Closed);
        Self::deposit_event(RawEvent::Close(bounty_id, remaining));
        Ok(())
    }

    pub fn assign_bounty_impl(
        bounty_id: BountyId,
        funder: T::AccountId,
        hunter: T::AccountId,
    ) -> DispatchResult {
        let bounty = Self::get_bounty(&bounty_id)?;
        Self::check_funder(&funder, &bounty)?;
        let state = Self::bounty_state_of(bounty_id);
        ensure!(
            (state == BountyState::Accepted) || (state == BountyState::Assigned), // could be assigned again
            Error::<T>::InvalidState
        );

        // judge new hunter is in hunting list
        ensure!(
            Self::hunting_for_bounty(bounty_id, &hunter),
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
    pub fn resolve_bounty_and_remark_impl(
        bounty_id: BountyId,
        funder: T::AccountId,
        remark: BountyRemarkCollaborationResult,
    ) -> DispatchResult {
        let bounty = Self::get_bounty(&bounty_id)?;
        Self::check_funder(&funder, &bounty)?;
        ensure!(
            Self::bounty_state_of(bounty_id) == BountyState::Submitted,
            Error::<T>::InvalidState
        );

        // TODO maybe other check

        // release currency
        let hunter = Self::hunted_for_bounty(&bounty_id);
        let (id, locked) = Self::parse_payment(&bounty);

        let fee = T::CouncilFee::get() * locked;
        let council_account = T::CouncilAccount::get();
        // todo may be use log to print remaining
        let _ = T::Currency::repatriate_reserved(
            id,
            &funder,
            &hunter,
            locked - fee,
            BalanceStatus::Free,
        )?;
        let _ = T::Currency::repatriate_reserved(
            id,
            &funder,
            &council_account,
            fee,
            BalanceStatus::Free,
        )?;

        // trigger
        T::BountyResolved::after_bounty_resolved(&bounty);

        Self::_add_reputation(&hunter, remark);
        Self::_add_mining_power(&bounty, fee, &funder, &hunter);

        Self::change_state(bounty_id, BountyState::Resolved);
        Self::deposit_event(RawEvent::Resolve(bounty_id));
        Self::deposit_event(RawEvent::FunderRemark(bounty_id, hunter, remark));
        // TODO maybe delete storage to save disk space

        Ok(())
    }

    fn _add_reputation(hunter: &T::AccountId, remark: BountyRemarkCollaborationResult) {
        T::ReputationBuilder::add_behavior_score_by_behavior(
            hunter,
            &Behavior::BountyResolve(BountyResolveCollaborationResult::Success),
        );
        T::ReputationBuilder::add_behavior_score_by_behavior(
            hunter,
            &Behavior::BountyRemark(remark),
        );
    }

    fn _add_mining_power(
        bounty: &BountyOf<T>,
        fee: BalanceOf<T>,
        funder: &T::AccountId,
        hunter: &T::AccountId,
    ) {
        let currency_id = Self::get_currency_id(bounty);
        let ratio = Self::currency_ratios(currency_id);

        let total_power = fee.saturated_into::<u128>() * ratio;
        let funder_power = total_power * 9 / 10;
        let hunter_power = total_power / 10;

        T::MiningPowerBuilder::add_mining_power(funder, funder_power);
        T::MiningPowerBuilder::add_mining_power(hunter, hunter_power);
        T::MiningPowerBuilder::add_session_total_mining_power(total_power);
    }
}
