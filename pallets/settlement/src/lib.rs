//! # Settlement Pallet
//!
//! 与 `docs/settlement-contract.md`（及 `cera-chain/docs/05-smart-contract/settlement-contract.md`）对应的结算模块。
//! 负责：收款方余额托管、平台费用计提、提现与日限额、事件与对账。

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
	pub mod pallet {
		use super::WeightInfo;
		use frame_support::{
			pallet_prelude::*,
			traits::{Currency, Get, ReservableCurrency},
		};
		use frame_system::pallet_prelude::*;
		use sp_runtime::traits::{Saturating, UniqueSaturatedInto, Zero};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// 结算用货币（提供 Balance 类型）
		type Currency: ReservableCurrency<Self::AccountId>;
		/// 每“天”对应的区块数，用于日限额重置
		type BlocksPerDay: Get<BlockNumberFor<Self>>;
		/// 结算模块资金账户（Treasury）
		type TreasuryAccount: Get<Self::AccountId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn payee_balance)]
	pub type PayeeBalance<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[derive(Clone, Encode, Decode, MaxEncodedLen, Default, PartialEq, Eq, TypeInfo, RuntimeDebug)]
	pub struct PayeeConfig<Balance> {
		pub active: bool,
		pub withdrawal_limit: Balance,
		pub daily_limit: Balance,
		pub fee_bps: u16,
	}

	#[pallet::storage]
	#[pallet::getter(fn payee_config)]
	pub type PayeeConfigs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		PayeeConfig<BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn platform_fee_balance)]
	pub type PlatformFeeBalance<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn daily_withdrawal)]
	pub type DailyWithdrawal<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_withdrawal_day)]
	pub type LastWithdrawalDay<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn paused)]
	pub type Paused<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Deposit {
			payee: T::AccountId,
			gross: BalanceOf<T>,
			fee: BalanceOf<T>,
		},
		SettlementRequested { payee: T::AccountId, amount: BalanceOf<T> },
		SettlementExecuted { payee: T::AccountId, amount: BalanceOf<T> },
		PayeeUpdated { payee: T::AccountId },
		PlatformFeeWithdrawn { to: T::AccountId, amount: BalanceOf<T> },
		Paused,
		Unpaused,
		EmergencyWithdrawn { to: T::AccountId, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		ModulePaused,
		PayeeNotActive,
		InsufficientBalance,
		ExceedsDailyLimit,
		Overflow,
		ExceedsWithdrawalLimit,
		ZeroAmount,
		ZeroBalance,
		NotPaused,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::deposit_for_payee())]
		pub fn deposit_for_payee(
			origin: OriginFor<T>,
			payee: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(!Paused::<T>::get(), Error::<T>::ModulePaused);
			ensure!(amount != Zero::zero(), Error::<T>::ZeroAmount);

			let fee_bps = PayeeConfigs::<T>::get(&payee).map(|c| c.fee_bps).unwrap_or(0);
			let ten_k = BalanceOf::<T>::from(10_000u32);
			let fee =
				amount / ten_k * BalanceOf::<T>::from(fee_bps as u32);
			let net = amount.saturating_sub(fee);

			PayeeBalance::<T>::try_mutate(&payee, |b| -> DispatchResult {
				*b = b.saturating_add(net);
				Ok(())
			})?;
			PlatformFeeBalance::<T>::try_mutate(|f| -> DispatchResult {
				*f = f.saturating_add(fee);
				Ok(())
			})?;

			Self::deposit_event(Event::Deposit { payee, gross: amount, fee });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::request_settlement())]
		pub fn request_settlement(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Paused::<T>::get(), Error::<T>::ModulePaused);
			ensure!(amount != Zero::zero(), Error::<T>::ZeroAmount);

			let config = PayeeConfigs::<T>::get(&who).ok_or(Error::<T>::PayeeNotActive)?;
			ensure!(config.active, Error::<T>::PayeeNotActive);
			ensure!(PayeeBalance::<T>::get(&who) >= amount, Error::<T>::InsufficientBalance);
			ensure!(amount <= config.withdrawal_limit, Error::<T>::ExceedsWithdrawalLimit);

			let block = frame_system::Pallet::<T>::block_number();
			let day: u64 = (block / T::BlocksPerDay::get()).unique_saturated_into();
			if LastWithdrawalDay::<T>::get(&who) != day {
				DailyWithdrawal::<T>::remove(&who);
				LastWithdrawalDay::<T>::insert(&who, day);
			}

			DailyWithdrawal::<T>::try_mutate(&who, |d| -> DispatchResult {
				let new_total: BalanceOf<T> = d
					.checked_add(&amount)
					.ok_or(Error::<T>::Overflow)?;
			
				ensure!(
					new_total <= config.daily_limit,
					Error::<T>::ExceedsDailyLimit
				);
			
				*d = new_total;
				Ok(())
			})?;
			Self::deposit_event(Event::SettlementRequested { payee: who, amount });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::execute_settlement())]
		pub fn execute_settlement(origin: OriginFor<T>, payee: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(!Paused::<T>::get(), Error::<T>::ModulePaused);

			let amount = PayeeBalance::<T>::get(&payee);
			ensure!(amount != Zero::zero(), Error::<T>::ZeroBalance);

			let zero_balance: BalanceOf<T> = Zero::zero();
			PayeeBalance::<T>::mutate(&payee, |b| *b = zero_balance);
			// 实际转账由链下或 runtime 侧配合 Treasury 完成；此处仅账本与事件
			Self::deposit_event(Event::SettlementExecuted { payee, amount });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::withdraw_platform_fee())]
		pub fn withdraw_platform_fee(origin: OriginFor<T>, to: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(!Paused::<T>::get(), Error::<T>::ModulePaused);

			let amount = PlatformFeeBalance::<T>::get();
			ensure!(amount != Zero::zero(), Error::<T>::ZeroAmount);

			let zero_balance: BalanceOf<T> = Zero::zero();
			PlatformFeeBalance::<T>::put(zero_balance);
			Self::deposit_event(Event::PlatformFeeWithdrawn { to, amount });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::set_payee_config())]
		pub fn set_payee_config(
			origin: OriginFor<T>,
			payee: T::AccountId,
			active: bool,
			withdrawal_limit: BalanceOf<T>,
			daily_limit: BalanceOf<T>,
			fee_bps: u16,
		) -> DispatchResult {
			ensure_root(origin)?;
			PayeeConfigs::<T>::insert(
				&payee,
				PayeeConfig {
					active,
					withdrawal_limit,
					daily_limit,
					fee_bps,
				},
			);
			Self::deposit_event(Event::PayeeUpdated { payee });
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::pause())]
		pub fn pause(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Paused::<T>::put(true);
			Self::deposit_event(Event::Paused);
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::unpause())]
		pub fn unpause(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			Paused::<T>::put(false);
			Self::deposit_event(Event::Unpaused);
			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::emergency_withdraw())]
		pub fn emergency_withdraw(
			origin: OriginFor<T>,
			to: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(Paused::<T>::get(), Error::<T>::NotPaused);
			Self::deposit_event(Event::EmergencyWithdrawn { to, amount });
			Ok(())
		}
	}
}

pub mod weights;
pub use weights::WeightInfo;
