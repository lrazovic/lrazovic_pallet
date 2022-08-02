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

    #[pallet::type_value]
    pub fn NeverStaked<T: Config>() -> T::BlockNumber {
        0_u32.into()
    }

    #[pallet::storage]
    #[pallet::getter(fn staked_times)]
    pub(super) type StakedTimes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, ValueQuery, NeverStaked<T>>;

    #[pallet::type_value]
    pub fn DefaultBlockTime<T: Config>() -> u32 {
        1_u32
    }

    #[pallet::type_value]
    pub fn DefaultPercentage<T: Config>() -> u8 {
        1_u8
    }

    #[pallet::storage]
    #[pallet::getter(fn percentage)]
    /// The percentage of "liquid" Token that a user receives after staking.
    /// X DOT gives (X + Percentage% X) of LDOT
    pub type Percentage<T> = StorageValue<_, u8, ValueQuery, DefaultPercentage<T>>;

    #[pallet::storage]
    #[pallet::getter(fn block_to_unlock)]
    /// The number of blocks that a user must wait before they can unstake.
    pub type BlockToUnlock<T> = StorageValue<_, u32, ValueQuery, DefaultBlockTime<T>>;

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
        TooFastStake,
        ZeroAmount,
        PercentageTooHigh,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn stake(origin: OriginFor<T>, #[pallet::compact] amount: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > 0_u8.into(), Error::<T>::ZeroAmount);
            ensure!(
                T::MainToken::free_balance(&who) >= amount.into(),
                Error::<T>::NotEnoughMainToken
            );

            let last_stake_time = <StakedTimes<T>>::get(&who);
            let now = <frame_system::Pallet<T>>::block_number();

            ensure!(
                now >= last_stake_time + BlockToUnlock::<T>::get().into(),
                Error::<T>::TooFastStake
            );

            // Reserve the `MainToken` token.
            let _ = T::MainToken::reserve(&who, amount.into());
            Self::deposit_event(Event::MainTokenStaked(who.clone(), amount));

            let value: u32 = Percentage::<T>::get().into();
            let staked_token_issued = amount.checked_add(value).unwrap_or(amount);

            // Issue new `StakedToken` tokens.
            // This is infallible, but doesn’t guarantee that the entire amount is issued, for example in the case of overflow.
            let issued = T::StakedToken::issue(staked_token_issued.into());
            Self::deposit_event(Event::StakedTokenIssued(staked_token_issued));

            // Deposit the `StakedToken` token to the user.
            let _ = T::StakedToken::resolve_into_existing(&who, issued);
            Self::deposit_event(Event::StakedTokenDeposited(
                who.clone(),
                staked_token_issued,
            ));
            let now = <frame_system::Pallet<T>>::block_number();
            <StakedTimes<T>>::insert(&who, now);

            Ok(())
        }

        #[pallet::weight(T::DbWeight::get().writes(1))]
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

            ensure!(
                now >= last_stake_time + BlockToUnlock::<T>::get().into(),
                Error::<T>::TooFastUnstake
            );

            // Withdraw the `StakedToken` tokens from the user.
            let _ = T::StakedToken::withdraw(
                &who,
                amount.into(),
                WithdrawReasons::RESERVE,
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenWithdrawn(who.clone(), amount));

            // Burn a `value` number StakedToken tokens.
            let _ = T::StakedToken::burn(amount.into());
            Self::deposit_event(Event::StakedTokenBurned(amount));

            // Remove the lock from `MainToken` tokens.
            T::MainToken::unreserve(&who, amount.into());
            Self::deposit_event(Event::MainTokenUnstaked(who, amount));

            Ok(())
        }

        #[pallet::weight(T::DbWeight::get().writes(1))]
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

            // Trasfer the `StakedToken` tokens from who to recv.
            let _ = T::StakedToken::transfer(
                &who,
                &recv,
                amount.into(),
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenTrasnferred(who, recv, amount));

            Ok(())
        }

        #[pallet::weight((0, Pays::Yes))]
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

        #[pallet::weight(0)]
        pub fn change_percentage(origin: OriginFor<T>, percentage: u8) -> DispatchResult {
            // In this way only the ROOT council can call the function!
            ensure_root(origin)?;

            ensure!(percentage <= 100, Error::<T>::PercentageTooHigh);

            Percentage::<T>::put(percentage);

            Ok(())
        }

        #[pallet::weight(0)]
        pub fn change_block_time(
            origin: OriginFor<T>,
            #[pallet::compact] block_time: u32,
        ) -> DispatchResult {
            // In this way only the ROOT council can call the function!
            ensure_root(origin)?;

            BlockToUnlock::<T>::put(block_time);

            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: T::BlockNumber) {
            if n % 100u32.into() == frame_support::sp_runtime::traits::Zero::zero() {
                // Do something every 100 blocks
                // TODO: From the POT send the tokens to the users
            };
        }
    }
}
