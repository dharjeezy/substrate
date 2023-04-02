// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use frame_support::{log, traits::OnRuntimeUpgrade};

pub mod v1 {
	use frame_support::pallet_prelude::*;

	use super::*;

	#[derive(Decode)]
	pub struct AssetDetailsV0<Balance, AccountId, DepositBalance> {
		pub owner: AccountId,
		pub issuer: AccountId,
		pub admin: AccountId,
		pub freezer: AccountId,
		pub supply: Balance,
		pub deposit: DepositBalance,
		pub min_balance: Balance,
		pub is_sufficient: bool,
		pub accounts: u32,
		pub sufficients: u32,
		pub approvals: u32,
		pub is_frozen: bool,
	}

	#[derive(Decode)]
	pub struct AssetDetailsV1<Balance, AccountId, DepositBalance> {
		pub owner: AccountId,
		pub issuer: AccountId,
		pub admin: AccountId,
		pub freezer: AccountId,
		pub supply: Balance,
		pub deposit: DepositBalance,
		pub min_balance: Balance,
		pub is_sufficient: bool,
		pub accounts: u32,
		pub sufficients: u32,
		pub approvals: u32,
		pub status: AssetStatus,
	}
}

pub mod v2 {
	use super::*;
	use crate::migration::v1::{AssetDetailsV0, AssetDetailsV1};
	use frame_support::{pallet_prelude::*, weights::Weight};

	impl<Balance, AccountId, DepositBalance> AssetDetailsV0<Balance, AccountId, DepositBalance> {
		fn migrate_from_v0_to_v2(self) -> AssetDetails<Balance, AccountId, DepositBalance> {
			let status = if self.is_frozen { AssetStatus::Frozen } else { AssetStatus::Live };

			AssetDetails {
				owner: Some(self.owner),
				issuer: Some(self.issuer),
				admin: Some(self.admin),
				freezer: Some(self.freezer),
				supply: self.supply,
				deposit: self.deposit,
				min_balance: self.min_balance,
				is_sufficient: self.is_sufficient,
				accounts: self.accounts,
				sufficients: self.sufficients,
				approvals: self.approvals,
				status,
			}
		}
	}

	impl<Balance, AccountId, DepositBalance> AssetDetailsV1<Balance, AccountId, DepositBalance> {
		fn migrate_from_v1_to_v2(self) -> AssetDetails<Balance, AccountId, DepositBalance> {
			AssetDetails {
				owner: Some(self.owner),
				issuer: Some(self.issuer),
				admin: Some(self.admin),
				freezer: Some(self.freezer),
				supply: self.supply,
				deposit: self.deposit,
				min_balance: self.min_balance,
				is_sufficient: self.is_sufficient,
				accounts: self.accounts,
				sufficients: self.sufficients,
				approvals: self.approvals,
				status: self.status,
			}
		}
	}

	pub struct MigrateToV2<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
		fn on_runtime_upgrade() -> Weight {
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();

			if onchain_version == 0 && current_version == 1 {
				let mut translated = 0u64;
				Asset::<T>::translate::<
					AssetDetailsV0<T::Balance, T::AccountId, DepositBalanceOf<T>>,
					_,
				>(|_key, old_value| {
					translated.saturating_inc();
					Some(old_value.migrate_from_v0_to_v2())
				});
				current_version.put::<Pallet<T>>();
				log::info!(target: "runtime::assets", "Upgraded {} pools, storage to version {:?}", translated, current_version);
				T::DbWeight::get().reads_writes(translated + 1, translated + 1)
			} else if onchain_version == 1 && current_version == 2 {
				let mut translated = 0u64;
				Asset::<T>::translate::<
					AssetDetailsV1<T::Balance, T::AccountId, DepositBalanceOf<T>>,
					_,
				>(|_key, old_value| {
					translated.saturating_inc();
					Some(old_value.migrate_from_v1_to_v2())
				});
				current_version.put::<Pallet<T>>();
				log::info!(target: "runtime::assets", "Upgraded {} pools, storage to version {:?}", translated, current_version);
				T::DbWeight::get().reads_writes(translated + 1, translated + 1)
			} else {
				log::info!(target: "runtime::assets",  "Migration did not execute. This probably should be removed");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			frame_support::ensure!(
				Pallet::<T>::on_chain_storage_version() == 0,
				"must upgrade linearly"
			);
			let prev_count = Asset::<T>::iter().count();
			Ok((prev_count as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), &'static str> {
			let prev_count: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);
			let post_count = Asset::<T>::iter().count() as u32;
			assert_eq!(
				prev_count, post_count,
				"the asset count before and after the migration should be the same"
			);

			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();

			frame_support::ensure!(current_version == 1, "must_upgrade");
			assert_eq!(
				current_version, onchain_version,
				"after migration, the current_version and onchain_version should be the same"
			);

			Ok(())
		}
	}
}
