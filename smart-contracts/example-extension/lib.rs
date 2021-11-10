#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;

/// This is an example of how an ink! contract may call the Substrate
/// runtime function `RandomnessCollectiveFlip::random_seed`. See the
/// file `runtime/chain-extension-example.rs` for that implementation.
///
/// Here we define the operations to interact with the Substrate runtime.
#[ink::chain_extension]
pub trait FetchRandom {
	type ErrorCode = RuntimeCallErr;

	#[ink(extension = 2)]
	fn call_pallet(key: &u8) -> Result<(u32, [u8; 32]), RuntimeCallErr>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RuntimeCallErr {
	FailToCallRuntime,
}

impl From<scale::Error> for RuntimeCallErr {
	fn from(_: scale::Error) -> Self {
		panic!("encountered unexpected invalid SCALE encoding")
	}
}

impl ink_env::chain_extension::FromStatusCode for RuntimeCallErr {
	fn from_status_code(status_code: u32) -> Result<(), Self> {
		match status_code {
			0 => Ok(()),
			1 => Err(Self::FailToCallRuntime),
			_ => panic!("encountered unknown status code"),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
	type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
	type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
	type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
	type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
	type RentFraction = <ink_env::DefaultEnvironment as Environment>::RentFraction;

	type ChainExtension = FetchRandom;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod rand_extension {
	use super::RuntimeCallErr;

	/// Defines the storage of our contract.
	#[ink(storage)]
	pub struct ExampleExtension {
		/// Stores a single `bool` value on the storage.
		value: [u8; 32],
		stored_number: u32,
	}

	#[ink(event)]
	pub struct NumUpdated {
		num: u32,
	}

	impl ExampleExtension {
		#[ink(constructor)]
		pub fn default() -> Self {
			Self { value: Default::default(), stored_number: Default::default() }
		}

		/// An example smart contract function demonstrating interactions originating in a custom pallet
		#[ink(message)]
		pub fn call_from_pallet(&mut self, value: u32) -> Result<(), RuntimeCallErr> {
			self.stored_number = value;
			self.env().emit_event(NumUpdated { num: value });
			Ok(())
		}

		/// Update a value given by argument
		#[ink(message)]
		pub fn send_to_pallet(&mut self, value: u8) -> Result<(),RuntimeCallErr> {
			self.env().extension().call_pallet(&value)?;
			Ok(())
		}

		/// Simply returns the current value.
		#[ink(message)]
		pub fn get(&self) -> [u8; 32] {
			self.value
		}
	}

	/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
	#[cfg(test)]
	mod tests {
		/// Imports all the definitions from the outer scope so we can use them here.
		use super::*;
		use ink_lang as ink;

		/// We test if the default constructor does its job.
		#[ink::test]
		fn default_works() {
			let rand_extension = ExampleExtension::default();
			assert_eq!(rand_extension.get(), [0; 32]);
		}

		#[ink::test]
		fn chain_extension_works() {
			struct MockedExtension;
			impl ink_env::test::ChainExtension for MockedExtension {
				/// The static function id of the chain extension.
				fn func_id(&self) -> u32 {
					2
				}

				/// The chain extension is called with the given input.
				///
				/// Returns an error code and may fill the `output` buffer with a
				/// SCALE encoded result. The error code is taken from the
				/// `ink_env::chain_extension::FromStatusCode` implementation for
				/// `RuntimeCallErr`.
				fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
					let ret: [u8; 32] = [1; 32];
					scale::Encode::encode_to(&ret, output);
					0
				}
			}
			ink_env::test::register_chain_extension(MockedExtension);
			let mut rand_extension = ExampleExtension::default();
			assert_eq!(rand_extension.get(), [0; 32]);

			// when
			rand_extension.update().expect("update must work");

			// then
			assert_eq!(rand_extension.get(), [1; 32]);
		}
	}
}
