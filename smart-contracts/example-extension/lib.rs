#![cfg_attr(not(feature = "std"), no_std)]

use ink_env::Environment;
use ink_lang as ink;
use ink_env::AccountId;

#[ink::chain_extension]
pub trait MyExtension {
	type ErrorCode = RuntimeCallErr;
	// Specify the function id. We will `match` on this in the runtime to map this to some custom pallet extrinsic
	#[ink(extension = 1)]
	fn do_store_in_map(key: u32) -> Result<(u32), RuntimeCallErr>;
	#[ink(extension = 2)]
	fn do_send_to_transfer(value: u32, recipient: AccountId) -> Result<(u32), RuntimeCallErr>;
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

	type ChainExtension = MyExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod rand_extension {
	use super::RuntimeCallErr;

	/// Defines the storage of our contract.
	#[ink(storage)]
	pub struct ExampleExtension {
		stored_number: u32,
	}

	#[ink(event)]
	pub struct UpdatedNum {
		result: u32,
	}

	impl ExampleExtension {
		#[ink(constructor)]
		pub fn default() -> Self {
			Self { stored_number: Default::default() }
		}

		/// A simple example function meant to demonstrate calling a smart contract with an argument from a custom pallet
		#[ink(message)]
		pub fn set_value(&mut self, value: u32) -> Result<(), RuntimeCallErr> {
			self.stored_number = value;
			self.env().emit_event(UpdatedNum { result: value });
			Ok(())
		}

		/// Increment the stored value by some addend. Specify a selector to make it easier to target this function from the extrinsic
		#[ink(message, selector = 0xABCDE)]
		pub fn add_to_value(&mut self, value: u32) -> Result<(), RuntimeCallErr> {
			self.stored_number += value;
			self.env().emit_event(UpdatedNum { result: self.stored_number });
			Ok(())
		}

		/// Receive the value and pass along to our extended custom pallet extrinsic
		#[ink(message)]
		pub fn store_in_map(&mut self, value: u32) -> Result<(), RuntimeCallErr> {
			self.env().extension().do_store_in_map(value)?;
			Ok(())
		}

		// Receive the value and pass to our extended transfer function
		#[ink(message)]
		pub fn send_to_transfer(
			&mut self,
			amount: u32,
			recipient: AccountId,
		) -> Result<(), RuntimeCallErr> {
			self.env().extension().do_send_to_transfer(amount, recipient)?;
			Ok(())
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
