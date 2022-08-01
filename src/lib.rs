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
    use frame_support::traits::{Currency, Get, ReservableCurrency};
    use frame_support::weights::Pays;
    use frame_system::pallet_prelude::*;

    type Balance = u32;

    // Allows easy access our Pallet's `Balance` type. Comes from `Currency` interface.
    // The balance of the `MainToken` type.
    type MainTokenBalance<T> =
        <<T as Config>::MainToken as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // Allows easy access our Pallet's `Balance` type. Comes from `Currency` interface.
    // The balance of the `StakedTokenBalance` type.
    type StakedTokenBalance<T> =
        <<T as Config>::StakedToken as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // TODO: Convert StakedTokenBalance<T> to MainTokenBalance<T>
    // impl<T: crate::Config> From <StakedTokenBalance<T>> for MainTokenBalance<T> {
    //     fn from(staked_token_balance: StakedTokenBalance<T>) -> MainTokenBalance<T> {
    //         staked_token_balance
    //     }
    // }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_democracy::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The "native" Token to stake
        type MainToken: ReservableCurrency<Self::AccountId>;

        /// The "liquid" Token given after staking
        type StakedToken: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type StakedTimes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event emitted when a MainToken is LOCKED by the owner. [from, amount]
        MainTokenStaked(T::AccountId, Balance),

        /// Event emitted when a MainToken is UNLOCKED by the owner. [from, amount]
        MainTokenUnstaked(T::AccountId, Balance),

        /// Event emitted when a StakedToken is DEPOSITED to the owner. [from, amount]
        StakedTokenDeposited(T::AccountId, Balance),

        /// Event emitted when a StakedToken is TRANSFERRED by the owner. [from, recv, amount]
        StakedTokenTrasnferred(T::AccountId, T::AccountId, Balance),

        /// Event emitted when a StakedToken is REMOVED from the owner. [from, amount]
        StakedTokenWithdrawn(T::AccountId, Balance),

        /// Event emitted when a StakedToken is ISSUED. [amount]
        StakedTokenIssued(Balance),

        /// Event emitted when a StakedToken is BURNED. [amount]
        StakedTokenBurned(Balance),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NotEnoughMainToken,
        NotEnoughStakedToken,
        NeverStaked,
        TransferToSelf,
        TooFastUnstake,
        ZeroAmount,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn stake(origin: OriginFor<T>, #[pallet::compact] amount: Balance) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            ensure!(amount > 0_u8.into(), Error::<T>::ZeroAmount);
            ensure!(
                T::MainToken::free_balance(&who) >= amount.into(),
                Error::<T>::NotEnoughMainToken
            );
            // Lock the `MainToken` token.
            let _ = T::MainToken::reserve(&who, amount.into());
            Self::deposit_event(Event::MainTokenStaked(who.clone(), amount.into()));
            // TODO: Handle errors.

            // Issue new `StakedToken` tokens.
            // This is infallible, but doesn’t guarantee that the entire amount is issued, for example in the case of overflow.

            // TODO: Issue value + NUMBER % of the amount.
            // TODO: NUMBER should be configurable by governance.
            let _ = T::StakedToken::issue(amount.into());
            Self::deposit_event(Event::StakedTokenIssued(amount));

            // Deposit the `StakedToken` token to the user.
            let _ = T::StakedToken::deposit_into_existing(&who, amount.into());
            Self::deposit_event(Event::StakedTokenDeposited(who.clone(), amount));
            let now = <frame_system::Pallet<T>>::block_number();
            let _ = <StakedTimes<T>>::insert(&who, now);

            // TODO: Handle errors.

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn unstake(origin: OriginFor<T>, #[pallet::compact] amount: Balance) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::ZeroAmount);

            ensure!(
                T::StakedToken::free_balance(&who) >= amount.into(),
                Error::<T>::NotEnoughStakedToken
            );

            let last_stake_time = <StakedTimes<T>>::get(&who);
            let now = <frame_system::Pallet<T>>::block_number();

            // // TODO: Change current_block >= block_number_staked to current_block >= block_number_staked + TIME_PERIOD_IN_BLOCKS
            ensure!(
                now > last_stake_time + 1_u8.into(),
                Error::<T>::TooFastUnstake
            );

            // Withdraw the `StakedToken` tokens from the user.
            let _ = T::StakedToken::withdraw(
                &who,
                amount.into(),
                WithdrawReasons::RESERVE,
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenWithdrawn(who.clone(), amount.into()));

            // Burn a `value` number StakedToken tokens.
            let _ = T::StakedToken::burn(amount.into());
            Self::deposit_event(Event::StakedTokenBurned(amount.into()));

            // Remove the lock from `MainToken` tokens.
            T::MainToken::unreserve(&who, amount.into());
            Self::deposit_event(Event::MainTokenUnstaked(who, amount.into()));

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn transfer(
            origin: OriginFor<T>,
            recv: T::AccountId,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            ensure!(who != recv, Error::<T>::TransferToSelf);

            ensure!(
                T::StakedToken::free_balance(&who) >= amount.into(),
                Error::<T>::NotEnoughStakedToken
            );

            // Withdraw the staked token from the user.
            let _ = T::StakedToken::transfer(
                &who,
                &recv,
                amount.into(),
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenTrasnferred(who, recv, amount.into()));

            Ok(())
        }

        #[pallet::weight((10_000, Pays::Yes))]
        pub fn create_proposal(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] weight: Balance,
        ) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin.clone())?;

            let staked_token_balance = T::StakedToken::total_balance(&who);
            ensure!(
                staked_token_balance >= weight.into(),
                Error::<T>::NotEnoughStakedToken
            );

            pallet_democracy::Pallet::<T>::propose(origin, proposal_hash, weight.into())?;

            // Free transaction if the extrinsic is executed correctly.
            Ok(Pays::No.into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote_in_favor(
            origin: OriginFor<T>,
            referendum_index: pallet_democracy::ReferendumIndex,
            #[pallet::compact] weight: Balance,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let _who = ensure_signed(origin.clone())?;

            // TODO: Check if it's correct
            let vote = pallet_democracy::AccountVote::Split {
                aye: weight.into(),
                nay: 0_u8.into(),
            };

            pallet_democracy::Pallet::<T>::vote(origin, referendum_index, vote)?;

            Ok(())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn vote_in_disfavour(
            origin: OriginFor<T>,
            referendum_index: pallet_democracy::ReferendumIndex,
            #[pallet::compact] weight: Balance,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let _who = ensure_signed(origin.clone())?;

            // TODO: Check if it's correct
            let vote = pallet_democracy::AccountVote::Split {
                aye: 0_u8.into(),
                nay: weight.into(),
            };

            let _ = pallet_democracy::Pallet::<T>::vote(origin, referendum_index, vote);

            Ok(())
        }

        // TODO: let who = ensure_root(origin.clone())?;
        // In this way only the ROOT council can call the function!
    }
}
