#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::DispatchResult;
    use frame_support::traits::tokens::{ExistenceRequirement, WithdrawReasons};
    use frame_support::traits::{Currency, Get, LockableCurrency};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Token to stake
        type MainToken: LockableCurrency<Self::AccountId>;

        /// The Staked Token
        type StakedToken: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // #[pallet::storage]
    // pub(super) type Pools<T: Config> =
    //    StorageMap<_, Blake2_128Concat, T::AccountId, (T::AccountId, T::BlockNumber), OptionQuery>;
    // TODO: Add a Storage to save the staking ids.

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when a claim is TRANSFERRED by the owner. [from, recv, claim]
        MainTokenStaked(T::AccountId, u32),
        MainTokenUnstaked(T::AccountId, u32),
        StakedTokenTrasnferred(T::AccountId, T::AccountId, u32),
        StakedTokenWithdrawn(T::AccountId, u32),
        StakedTokenDeposited(T::AccountId, u32),
        StakedTokenIssued(u32),
        StakedTokenBurned(u32),
        // TODO: Add more events here.
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NotEnoughMainToken,
        NotEnoughStakedToken,
        // TODO: Add more errors here.
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn stake(origin: OriginFor<T>, value: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            let id: [u8; 8] = [1; 8];
            // TODO: Create a REAL unique ID.
            // Maybe PUT the id in the storage?

            // Lock the `MainToken` token.
            T::MainToken::set_lock(id, &who, value.into(), WithdrawReasons::RESERVE);
            Self::deposit_event(Event::MainTokenStaked(who.clone(), value));
            // TODO: Handle errors.

            // Issue new `StakedToken` tokens.
            // This is infallible, but doesn’t guarantee that the entire amount is issued, for example in the case of overflow.
            let _ = T::StakedToken::issue(value.into());
            Self::deposit_event(Event::StakedTokenIssued(value));

            // Deposit the `StakedToken` token to the user.
            let _ = T::StakedToken::deposit_into_existing(&who, value.into());
            Self::deposit_event(Event::StakedTokenDeposited(who, value));
            // TODO: Handle errors.

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unstake(origin: OriginFor<T>, value: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            let id: [u8; 8] = [1; 8];
            // TODO: Use a REAL unique ID.
            // Maybe GET the id from the storage?

            // Withdraw the `StakedToken` tokens from the user.
            let _ = T::StakedToken::withdraw(
                &who,
                value.into(),
                WithdrawReasons::RESERVE,
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenWithdrawn(who.clone(), value));

            // Burn a `value` number StakedToken tokens.
            let _ = T::StakedToken::burn(value.into());
            Self::deposit_event(Event::StakedTokenBurned(value));

            // Remove the lock from `MainToken` tokens.
            T::MainToken::remove_lock(id, &who);
            Self::deposit_event(Event::MainTokenUnstaked(who, value));

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn transfer(origin: OriginFor<T>, recv: T::AccountId, value: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            if who == recv {
                // no change needed
                return Ok(());
            }

            // Withdraw the staked token from the user.
            let _ = T::StakedToken::transfer(
                &who,
                &recv,
                value.into(),
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenTrasnferred(who, recv, value));

            Ok(())
        }
    }
}
