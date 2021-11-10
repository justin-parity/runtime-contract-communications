#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::inherent::Vec;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Currency, weights::Weight};
	use frame_system::pallet_prelude::*;
	use pallet_contracts::chain_extension::UncheckedFrom;

	use log::info;

	type BalanceOf<T> = <<T as pallet_contracts::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_contracts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_value)]
	pub(super) type ContractEntry<T> = StorageValue<_, u32, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CalledContract(T::AccountId),
		ContractCallFailed(DispatchError),
		CalledPalletFromContract(u32)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		SmartContractCallError,
		/// Value given by smart contract is already set
		ValueAlreadyExists
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash>,
		T::AccountId: AsRef<[u8]>,
	{
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// An example to demonstrate calling a smart contract from an extrinsic
		pub fn call_smart_contract(
			origin: OriginFor<T>,
			dest: T::AccountId,
			selector: Vec<u8>,
			arg: u32,
		) -> DispatchResult {

			
			let who = ensure_signed(origin)?;
			// Amount to transfer
			let value: BalanceOf<T> = Default::default();
			// Arbitrary gas limit
			let gas_limit = 1000;

			// // log values to compare
			// info!("{:?}", selector); // [122, 20, 161, 130]
			// info!("{:?}", arg); // 42

			// see https://github.com/paritytech/substrate/blob/a9465729e2c5d2ef8d87ac404da27e5e10adde8a/frame/contracts/src/benchmarking/mod.rs#L2264-L2268
			let data = (selector, arg).encode();

			// info!("{:?}", data); // [16, 122, 20, 161, 130, 42, 0, 0, 0]     

			pallet_contracts::Pallet::<T>::bare_call(
				who,
				dest.clone(),
				value,
				gas_limit,
				data,
				true,
			)
			.result?;

			// do send event
			// Self::deposit_event(Event::CalledContract(success));

			Ok(())
		}

		// TODO: Less generic extrinsic for calling flipper
	}

	impl <T: Config> Pallet<T> {
		// An example pallet function to demonstrate calling from a smart contract
		pub fn call_from_contract(
			val: u32,
		) -> DispatchResult {
			info!("in extrinsic received {}", val);

			// Do something with the value
			ensure!(!(ContractEntry::<T>::get() == val), Error::<T>::ValueAlreadyExists);
			ContractEntry::<T>::put(val);
			Self::deposit_event(Event::CalledPalletFromContract(val));
			Ok(())
		}
	}
}
