#![cfg_attr(not(feature = "std"), no_std)]

mod types;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    traits::{Currency, EnsureOrigin},
    IterableStorageMap,
};
use frame_system::{ensure_signed, RawOrigin};

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;

    type CouncilOrigin: EnsureOrigin<Self::Origin>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {

    }
}
decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>, {
        Tmp(AccountId, Balance),
   }
);
decl_storage! {
    trait Store for Module<T: Trait> as OSBounties {

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
    }
}

impl<T: Trait> Module<T> {}
