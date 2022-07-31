use crate as simple_pool;
use frame_support::traits::EqualPrivilegeOnly;
use frame_support::traits::SortedMembers;
use frame_support::traits::{ConstU16, ConstU32, ConstU64};
use frame_support::{ord_parameter_types, parameter_types};
use frame_system as system;
use frame_system::EnsureRoot;
use frame_system::EnsureSignedBy;
use pallet_democracy;
use pallet_scheduler;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        TemplateModule: simple_pool::{Pallet, Call, Storage, Event<T>},
        Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
        Democracy: pallet_democracy::{Pallet, Call, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
    }
);

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_scheduler::Config for Test {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = ();
    type ScheduleOrigin = EnsureRoot<u64>;
    type MaxScheduledPerBlock = ();
    type WeightInfo = ();
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type PreimageProvider = ();
    type NoPreimagePostponement = ();
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

// impl pallet_balances::Config for Test {
// 	type MaxReserves = ();
// 	type ReserveIdentifier = [u8; 8];
// 	type MaxLocks = ConstU32<10>;
// 	type Balance = u64;
// 	type Event = Event;
// 	type DustRemoval = ();
// 	type ExistentialDeposit = ConstU64<1>;
// 	type AccountStore = System;
// 	type WeightInfo = ();
// }

parameter_types! {
    pub static PreimageByteDeposit: u64 = 0;
    pub static InstantAllowed: bool = false;
}

ord_parameter_types! {
    pub const One: u64 = 1;
    pub const Two: u64 = 2;
    pub const Three: u64 = 3;
    pub const Four: u64 = 4;
    pub const Five: u64 = 5;
    pub const Six: u64 = 6;
}

pub struct OneToFive;
impl SortedMembers<u64> for OneToFive {
    fn sorted_members() -> Vec<u64> {
        vec![1, 2, 3, 4, 5]
    }
    #[cfg(feature = "runtime-benchmarks")]
    fn add(_m: &u64) {}
}

impl pallet_democracy::Config for Test {
    type Proposal = Call;
    type Event = Event;
    type Currency = pallet_balances::Pallet<Self>; // TODO: Modify this to use the STAKED balance.
    type EnactmentPeriod = ConstU64<2>;
    type LaunchPeriod = ConstU64<2>;
    type VotingPeriod = ConstU64<2>;
    type VoteLockingPeriod = ConstU64<3>;
    type FastTrackVotingPeriod = ConstU64<2>;
    type MinimumDeposit = ConstU64<1>;
    type ExternalOrigin = EnsureSignedBy<Two, u64>;
    type ExternalMajorityOrigin = EnsureSignedBy<Three, u64>;
    type ExternalDefaultOrigin = EnsureSignedBy<One, u64>;
    type FastTrackOrigin = EnsureSignedBy<Five, u64>;
    type CancellationOrigin = EnsureSignedBy<Four, u64>;
    type BlacklistOrigin = EnsureRoot<u64>;
    type CancelProposalOrigin = EnsureRoot<u64>;
    type VetoOrigin = EnsureSignedBy<OneToFive, u64>;
    type CooloffPeriod = ConstU64<2>;
    type PreimageByteDeposit = PreimageByteDeposit;
    type Slash = ();
    type InstantOrigin = EnsureSignedBy<Six, u64>;
    type InstantAllowed = InstantAllowed;
    type Scheduler = Scheduler;
    type MaxVotes = ConstU32<100>;
    type OperationalPreimageOrigin = EnsureSignedBy<Six, u64>;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = ();
    type MaxProposals = ConstU32<100>;
}

impl simple_pool::Config for Test {
    type Event = Event;
    type MainToken = Balances;
    type StakedToken = Balances;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(1, 10), (2, 10), (3, 10), (4, 10), (5, 2)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
