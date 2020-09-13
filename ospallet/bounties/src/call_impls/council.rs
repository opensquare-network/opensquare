use frame_support::{dispatch::DispatchResult, ensure};

use opensquare_primitives::BountyId;
use orml_traits::MultiReservableCurrency;

use crate::types::BountyState;
use crate::types::CloseReason;
use crate::{Error, Module, RawEvent, Trait};

impl<T: Trait> Module<T> {
    pub fn examine_bounty_impl(bounty_id: BountyId, accepted: bool) -> DispatchResult {
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

    pub fn force_close_bounty_impl(bounty_id: BountyId, reason: CloseReason) -> DispatchResult {
        let bounty = Self::get_bounty(&bounty_id)?;
        let funder = Self::get_funder(&bounty);
        let (id, locked) = Self::parse_payment(&bounty);
        // remove hunter for a bounty
        Self::remove_hunters_for_bounty(bounty_id);
        Self::check_bounty_can_be_closed(bounty_id)?;
        // release reserved balance, todo maybe use log to print it
        let remaining = T::Currency::unreserve(id, &funder, locked);

        Self::change_state(bounty_id, BountyState::CouncilClosed);
        Self::deposit_event(RawEvent::ForceClosed(bounty_id, reason, remaining));

        Ok(())
    }
}
