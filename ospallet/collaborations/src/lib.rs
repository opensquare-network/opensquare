#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::EnsureOrigin,
};

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type CouncilOrigin: EnsureOrigin<Self::Origin>;
}
decl_error! {
    pub enum Error for Module<T: Trait> {

    }
}
decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId
    {
        Holder(AccountId),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as OSCollaborations {

    }
}
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;
    }
}
