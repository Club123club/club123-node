//! Weights for pallet_settlement.
//! Can be replaced with benchmark-generated weights later.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn deposit_for_payee() -> Weight;
	fn request_settlement() -> Weight;
	fn execute_settlement() -> Weight;
	fn withdraw_platform_fee() -> Weight;
	fn set_payee_config() -> Weight;
	fn pause() -> Weight;
	fn unpause() -> Weight;
	fn emergency_withdraw() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn deposit_for_payee() -> Weight {
		Weight::from_parts(30_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn request_settlement() -> Weight {
		Weight::from_parts(35_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	fn execute_settlement() -> Weight {
		Weight::from_parts(25_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn withdraw_platform_fee() -> Weight {
		Weight::from_parts(20_000_000, 0)
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn set_payee_config() -> Weight {
		Weight::from_parts(25_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1))
	}
	fn pause() -> Weight {
		Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1))
	}
	fn unpause() -> Weight {
		Weight::from_parts(10_000_000, 0).saturating_add(T::DbWeight::get().writes(1))
	}
	fn emergency_withdraw() -> Weight {
		Weight::from_parts(15_000_000, 0)
	}
}

impl WeightInfo for () {
	fn deposit_for_payee() -> Weight { Weight::from_parts(30_000_000, 0) }
	fn request_settlement() -> Weight { Weight::from_parts(35_000_000, 0) }
	fn execute_settlement() -> Weight { Weight::from_parts(25_000_000, 0) }
	fn withdraw_platform_fee() -> Weight { Weight::from_parts(20_000_000, 0) }
	fn set_payee_config() -> Weight { Weight::from_parts(25_000_000, 0) }
	fn pause() -> Weight { Weight::from_parts(10_000_000, 0) }
	fn unpause() -> Weight { Weight::from_parts(10_000_000, 0) }
	fn emergency_withdraw() -> Weight { Weight::from_parts(15_000_000, 0) }
}
