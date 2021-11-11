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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Currency, inherent::Vec};
	use frame_system::pallet_prelude::*;
	use pallet_contracts::chain_extension::UncheckedFrom;

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

	const MAX_LENGTH:usize = 50;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_items)]
	pub(super) type ContractEntry<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CalledContract,
		CalledPalletFromContract(Vec<u8>)
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ValueAlreadyExists,
		NewValueTooLarge
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
			// selector as given in the metadata.json file of the compiled contract
			selector: Vec<u8>,
			arg: u32,
			#[pallet::compact] gas_limit: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(selector.len() < MAX_LENGTH, Error::<T>::NewValueTooLarge);
			// Amount to transfer
			let value: BalanceOf<T> = Default::default();

			// data argument is expected to be encoded vector of selector + any args
			let data = (selector, arg).encode();
			pallet_contracts::Pallet::<T>::bare_call(
				who,
				dest.clone(),
				value,
				gas_limit,
				data,
				true,
			)
			.result?;

			Self::deposit_event(Event::CalledContract);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		// A less generic example, demonstrating the selector. Invokes a smart contract method that toggles a boolean in storage
		pub fn flip_smart_contract(origin: OriginFor<T>, dest: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let to_transfer: BalanceOf<T> = Default::default();
			let gas_limit = 10000;
			let selector = 0x000abcde;
			pallet_contracts::Pallet::<T>::bare_call(
				who,
				dest.clone(),
				to_transfer,
				gas_limit,
				[selector].encode(),
				true,
			)
			.result?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]		
		// An example extrinsic to demonstrate calling from a smart contract
		pub fn insert_number(
			origin: OriginFor<T>,
			// val: BoundedVec<u8, T::MaxLength>,
			val: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// Do something with the value
			ContractEntry::<T>::insert(who, val.to_vec());
			Self::deposit_event(Event::CalledPalletFromContract(val.to_vec()));
			Ok(())
		}
	}
}
