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

    use frame_support::sp_runtime::traits::AccountIdConversion;
    use frame_support::PalletId;
    use frame_system::pallet_prelude::*;

    // Allows easy access our Pallet's `Balance` type. Comes from `Currency` interface.
    // The balance of the `StakedTokenBalance` type.
    type StakedTokenBalance<T> =
        <<T as Config>::StakedToken as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    type Balance = u128;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_democracy::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The "native" Token to stake
        type MainToken: ReservableCurrency<Self::AccountId, Balance = Balance>;

        /// The "liquid" Token given after staking
        type StakedToken: Currency<Self::AccountId, Balance = Balance>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn staked_times)]
    pub(super) type StakedTimes<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, OptionQuery>;

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
        StakedTokenTransferred(T::AccountId, T::AccountId, StakedTokenBalance<T>),

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
        /// An account is trying to stake more than it has.
        NotEnoughMainToken,

        /// An account is trying to unstake more than it has.
        NotEnoughStakedToken,

        /// An account is trying to transfer funds to itself.
        TransferToSelf,

        /// An account is trying to unstake without waiting the required number of blocks.
        TooFastUnstake,

        /// An account is trying to stake again without waiting the required number of blocks.
        TooFastStake,

        /// An account is trying to stake/unstake/stake a 0 amount of tokens.
        ZeroAmount,

        /// The governance is trying to set a value that is > 100%.
        PercentageTooHigh,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig;

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            // Create POT account
            let pallet_account_id: T::AccountId = T::PalletId::get().into_account_truncating();
            let amount = 1_000_000_000;
            let _ = T::StakedToken::issue(amount);
            if T::StakedToken::free_balance(&pallet_account_id) < amount {
                let _ = T::StakedToken::make_free_balance_be(&pallet_account_id, amount);
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn stake(origin: OriginFor<T>, amount: Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > 0_u8.into(), Error::<T>::ZeroAmount);

            ensure!(
                T::MainToken::free_balance(&who) >= amount,
                Error::<T>::NotEnoughMainToken
            );

            let last_stake_time = <StakedTimes<T>>::get(&who).unwrap_or_else(|| 0_u8.into());
            let now = <frame_system::Pallet<T>>::block_number();

            ensure!(
                now >= last_stake_time + BlockToUnlock::<T>::get().into(),
                Error::<T>::TooFastStake
            );

            // Reserve the `MainToken` token.
            let _ = T::MainToken::reserve(&who, amount);
            Self::deposit_event(Event::MainTokenStaked(who.clone(), amount));

            let value: u32 = Percentage::<T>::get().into();
            let staked_token_issued = amount.checked_add(value.into()).unwrap_or(amount);

            // Issue new `StakedToken` tokens.
            // This is infallible, but doesn’t guarantee that the entire amount is issued, for example in the case of overflow.
            let issued = T::StakedToken::issue(staked_token_issued);
            Self::deposit_event(Event::StakedTokenIssued(staked_token_issued));

            // Deposit the `StakedToken` token to the user.
            T::StakedToken::resolve_creating(&who, issued);
            Self::deposit_event(Event::StakedTokenDeposited(
                who.clone(),
                staked_token_issued,
            ));

            // Set the block_time in which the operation is performed.
            let now = <frame_system::Pallet<T>>::block_number();
            <StakedTimes<T>>::insert(&who, now);

            Ok(())
        }

        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn unstake(origin: OriginFor<T>, amount: Balance) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::ZeroAmount);

            ensure!(
                T::StakedToken::free_balance(&who) >= amount,
                Error::<T>::NotEnoughStakedToken
            );

            let last_stake_time = <StakedTimes<T>>::get(&who).unwrap_or_else(|| 0_u8.into());
            let now = <frame_system::Pallet<T>>::block_number();

            ensure!(
                now >= last_stake_time + BlockToUnlock::<T>::get().into(),
                Error::<T>::TooFastUnstake
            );

            // Withdraw the `StakedToken` tokens from the user.
            let _ = T::StakedToken::withdraw(
                &who,
                amount,
                WithdrawReasons::RESERVE,
                ExistenceRequirement::KeepAlive,
            );
            Self::deposit_event(Event::StakedTokenWithdrawn(who.clone(), amount));

            // Burn a `value` number StakedToken tokens.
            let _ = T::StakedToken::burn(amount);
            Self::deposit_event(Event::StakedTokenBurned(amount));

            // Remove the lock from `MainToken` tokens.
            T::MainToken::unreserve(&who, amount);
            Self::deposit_event(Event::MainTokenUnstaked(who.clone(), amount));

            // Remove the last_block_time value from the map.
            <StakedTimes<T>>::remove(&who);

            Ok(())
        }

        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn transfer(
            origin: OriginFor<T>,
            recv: T::AccountId,
            #[pallet::compact] amount: StakedTokenBalance<T>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let who = ensure_signed(origin)?;

            ensure!(who != recv, Error::<T>::TransferToSelf);

            ensure!(
                T::StakedToken::free_balance(&who) >= amount,
                Error::<T>::NotEnoughStakedToken
            );

            ensure!(amount > 0, Error::<T>::ZeroAmount);

            // Trasfer the `StakedToken` tokens from who to recv.
            let _ = T::StakedToken::transfer(&who, &recv, amount, ExistenceRequirement::KeepAlive);
            Self::deposit_event(Event::StakedTokenTransferred(who, recv, amount));

            Ok(())
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
            if n % 10u8.into() == frame_support::sp_runtime::traits::Zero::zero() {
                for who in <StakedTimes<T>>::iter_keys() {
                    if <StakedTimes<T>>::get(&who).is_some() {
                        let pot_address = Self::account_id();
                        let _ = T::StakedToken::transfer(
                            &pot_address,
                            &who,
                            777,
                            ExistenceRequirement::KeepAlive,
                        );
                        Self::deposit_event(Event::StakedTokenTransferred(pot_address, who, 777));
                    }
                }
            };
        }
    }

    impl<T: Config> Pallet<T> {
        /// get voting pot address to deposit slashed tokens to and take rewards from
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
