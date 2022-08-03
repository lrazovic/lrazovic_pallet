# Simple Pool - PBA Cambridge 2022

[![Check, Test and Clippy](https://github.com/lrazovic/lrazovic_pallet/actions/workflows/check-and-lint.yaml/badge.svg)](https://github.com/lrazovic/lrazovic_pallet/actions/workflows/check-and-lint.yaml)

## Idea

The aim of this project is to create a pallet to manage a liquidity pool. A user can stake a token (e.g. a DOT) and receive a liquid token (e.g. a LDOT). The user can:

- Trade/transfer the LDOT
- Hold it to receive more LDOTs every X blocks
- Propose and vote changes about economic paramenters of the pool.

> **Note** <br>
> What is a liquidity pool? A liquidity pool is a digital pile of cryptocurrency locked in a smart contract. This results in creating liquidity for faster transactions. A major component of a liquidity pool are automated market makers (AMMs). An AMM is a protocol that uses liquidity pools to allow digital assets to be traded in an automated way rather than through a traditional market of buyers and sellers.
>
> [Source](https://www.coindesk.com/learn/what-are-liquidity-pools/)

> **Warning** <br>
> This is an educational purpose project only. Should NOT be used in a real production system.

## Build and test (as a single pallet)

1. Run tests with `cargo test`
2. Build with `cargo build --release`

## Build and test

1. Clone [this modified](https://github.com/lrazovic/substrate-node) version of `substrate-node-template`.
2. Build with `cargo build --release`
3. Run the binary with `./target/release/node-template --dev`
4. Interact with the node using [polkadot.js](https://polkadot.js.org/apps/) and/or with [this modified](https://github.com/lrazovic/substrate-front-end) version of `substrate-front-end-template` (Useful to see the balance in LDOT).

## Anatomy of the pallet

The following is a summary of `src/lib.rs`

### Exposed extrinsics

- `stake(amount: u128)`
- `unstake(amount: u128)`
- `transfer(recv: T::AccountId, amount: u128)`
- `change_percentage(percentage: u8)`
- `change_block_time(block_time: u32)`

### Hooks

- `on_finalize()`

### Storage

- `BlockToUnlock<T> = StorageValue<_, u32, ValueQuery, DefaultBlockTime<T>>`
- `Percentage<T> = StorageValue<_, u8, ValueQuery, DefaultPercentage<T>>`
- `StakedTimes<T> = StorageMap<_, Blake2_128Concat, T::AccountId, T::BlockNumber, OptionQuery>`

### Config

- `type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>`
- `type MainToken: ReservableCurrency<Self::AccountId, Balance = u128>`
- `type StakedToken: Currency<Self::AccountId, Balance = u128>`
- `type PalletId: Get<PalletId>`

## Implementation and simplifications

+ Instead of sending the funds via `pallet-staking` I used a `ReservableCurrency` to handle the "main token", so I can do a `reserve` to lock the funds and give a `Currency` representing the Liquid Token in return.
+ When the user calls `stake(amount)` an `amount` of `ReservableCurrency` is reserved and a `(amount + n%)` of `Currency` is created and deposited to the user.
+ The user cannot call `stake(amount)` again before a number of blocks (`BlockToUnlock`).
+ The user cannot call `unstake(amount)` before a number of blocks (`BlockToUnlock`).
+ After a number of blocks (`BlockToUnlock`) the user can call `unstake(amount)` to burn an `amount` of `Currency` and unreserve his portion of `ReservableCurrency`.
+ The user can transfer using `ransfer(recv, amount)` part of his `Currency`
+ Using `pallet_democracy` the user can create a proposal paying in `Currency` (so the liquid token, not in `ReservableCurrency`), as [shown here](https://github.com/lrazovic/substrate-node/blob/main/runtime/src/lib.rs#L361).
+ Through governance then users holding the liquid token can vote to use `change_percentage(percentage)` and `change_block_time(block_time)` to vary the economic parameters of the pool.
+ As an incentive not to transfer liquid tokens, new tokens are issued every X blocks and distributed to users using logic inside the `on_finalize` hook.

> **Warning** <br>
> Of course, I am aware that it makes no economic sense, but I had fun experimenting by combining various pallets.

## Future improvements

- Use [Cumulus](https://github.com/paritytech/cumulus) to convert a Substrate FRAME runtime into a Parachain runtime.
- Use XCM and XCMP to transfer the LDOT to other parachains.
- Build a frontend that queries the DOT and LDOT balances
