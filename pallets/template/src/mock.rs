use crate as pallet_template;
use frame_system::{self as system};
use sp_core::H256;
use sp_runtime::{AccountId32,
	testing::Header, traits::{BlakeTwo256,
		IdentityLookup,
	}};
use frame_support::{parameter_types, weights::{Weight, IdentityFee}};
use pallet_balances;
use pallet_randomness_collective_flip;
use pallet_timestamp;
use pallet_utility;
use std::cell::RefCell;
use pallet_transaction_payment::CurrencyAdapter;
use pallet_contracts::{
	chain_extension::{
		ChainExtension, Environment, Ext, InitState, Result as ExtensionResult, RetVal,
		ReturnFlags, SysConfig, UncheckedFrom,
	},
	Schedule,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{

		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		TemplateModule: pallet_template::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
		Contracts: pallet_contracts::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Storage, Event},
	}
);

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

pub type Balance = u64;

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	// type AccountId = u64;
	// type AccountId = AccountId;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

// impl system::Config for Test {
// 	type AccountData = pallet_balances::AccountData<Balance>;
// 	type BlockHashCount = BlockHashCount;
// 	type BlockNumber = BlockNumber;
// 	type Event = Event;
// 	type Hash = H256;
// 	type Hashing = BlakeTwo256;
// 	type Header = Header;
// 	type Lookup = IdentityLookup<Self::AccountId>;
// 	type OnKilledAccount = ();
// 	type OnNewAccount = ();
// 	type OnSetCode = ();
// 	type PalletInfo = PalletInfo;
// 	type SS58Prefix = SS58Prefix;
// 	type SystemWeightInfo = ();
// 	type Version = ();
// }

parameter_types! {
	// pub const BlockHashCount: u64 = 250;
	// pub BlockWeights: frame_system::limits::BlockWeights =
	// 	frame_system::limits::BlockWeights::simple_max(2 * WEIGHT_PER_SECOND);
	pub static ExistentialDeposit: u64 = 0;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	// type AccountStore = Test;
	type AccountStore = System;
	// type AccountStore = frame_system::Pallet<Test>;
	
	type WeightInfo = ();
}

impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

/// A filter whose filter function can be swapped at runtime.
pub struct TestFilter;

thread_local! {
	static CALL_FILTER: RefCell<fn(&Call) -> bool> = RefCell::new(|_| true);
}

impl TestFilter {
	pub fn set_filter(filter: fn(&Call) -> bool) {
		CALL_FILTER.with(|fltr| *fltr.borrow_mut() = filter);
	}
}

thread_local! {
	static TEST_EXTENSION: RefCell<TestExtension> = Default::default();
}

pub struct TestExtension {
	enabled: bool,
	last_seen_buffer: Vec<u8>,
	last_seen_inputs: (u32, u32, u32, u32),
}

impl TestExtension {
	fn disable() {
		TEST_EXTENSION.with(|e| e.borrow_mut().enabled = false)
	}

	fn last_seen_buffer() -> Vec<u8> {
		TEST_EXTENSION.with(|e| e.borrow().last_seen_buffer.clone())
	}

	fn last_seen_inputs() -> (u32, u32, u32, u32) {
		TEST_EXTENSION.with(|e| e.borrow().last_seen_inputs.clone())
	}
}

impl Default for TestExtension {
	fn default() -> Self {
		Self { enabled: true, last_seen_buffer: vec![], last_seen_inputs: (0, 0, 0, 0) }
	}
}

impl ChainExtension<Test> for TestExtension
where
	Test: SysConfig + pallet_contracts::Config,
	// <Test as SysConfig>::AccountId: UncheckedFrom<<Test as SysConfig>::Hash> + AsRef<[u8]>,
	<Test as SysConfig>::AccountId: UncheckedFrom<<Test as SysConfig>::Hash>,
{
	fn call<E: Ext>(func_id: u32, env: Environment<E, InitState>) -> ExtensionResult<RetVal>
	where
		E: Ext<T = Test>,
		// <E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash>,
	{
		match func_id {
			0 => {
				let mut env = env.buf_in_buf_out();
				let input = env.read(2)?;
				env.write(&input, false, None)?;
				TEST_EXTENSION.with(|e| e.borrow_mut().last_seen_buffer = input);
				Ok(RetVal::Converging(func_id))
			},
			1 => {
				let env = env.only_in();
				TEST_EXTENSION.with(|e| {
					e.borrow_mut().last_seen_inputs =
						(env.val0(), env.val1(), env.val2(), env.val3())
				});
				Ok(RetVal::Converging(func_id))
			},
			2 => {
				let mut env = env.buf_in_buf_out();
				let weight = env.read(2)?[1].into();
				env.charge_weight(weight)?;
				Ok(RetVal::Converging(func_id))
			},
			3 => Ok(RetVal::Diverging { flags: ReturnFlags::REVERT, data: vec![42, 99] }),
			_ => {
				panic!("Passed unknown func_id to test chain extension: {}", func_id);
			},
		}
	}

	fn enabled() -> bool {
		TEST_EXTENSION.with(|e| e.borrow().enabled)
	}
}

parameter_types! {
	pub const ContractDeposit: u64 = 16;
	pub const DeletionQueueDepth: u32 = 1024;
	pub const DeletionWeightLimit: Weight = 500_000_000_000;
	pub MySchedule: Schedule<Test> = <Schedule<Test>>::default();
}

impl pallet_randomness_collective_flip::Config for Test {}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = Randomness;
	type Currency = Balances;
	type Event = Event;
	type Call = Call;
	type CallFilter = frame_support::traits::Nothing;
	type ContractDeposit = ContractDeposit;
	type CallStack = [pallet_contracts::Frame<Self>; 31];
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = ();
	type ChainExtension = TestExtension;
	type DeletionQueueDepth = DeletionQueueDepth;
	type DeletionWeightLimit = DeletionWeightLimit;
	type Schedule = MySchedule;
}

impl pallet_template::Config for Test {
	type Event = Event;
	type Currency = Balances;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
impl pallet_utility::Config for Test {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxValueSize: u32 = 16_384;
	pub const MaxCodeSize: u32 = 2 * 1024;
	pub const TransactionByteFee: u64 = 0;
}

// // Build genesis storage according to the mock runtime.
// pub fn new_test_ext() -> sp_io::TestExternalities {
// 	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
// }
