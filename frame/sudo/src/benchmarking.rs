// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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

//! Sudo pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use sp_runtime::DispatchError;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	sudo {
		let c in 0 .. 16_000;

		// set up an account
		let caller: T::AccountId = whitelisted_caller();

		// put caller into storage because of the key variable which is storage write
		Key::<T>::put(caller.clone()); // puts caller into storage so that it will be sudo

		let bytes = vec![0u8, c as u8];

		//has no tangible weights and can be used for benchmarking
		let noop = frame_system::Call::<T>::remark{remark: bytes};

	}: _(RawOrigin::Signed(caller.clone()), Box::new(noop.into()))
	verify {
		// check sudid event fired to see if it works
		assert_last_event::<T>(Event::Sudid { sudo_result: Ok(()) }.into());
	}
}

/*#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{Test, new_test_ext};
	use frame_support::assert_ok;

	#[test]
	fn sudo() {
		new_test_ext(1).execute_with(|| {
			assert_ok!(test_benchmark_sudo::<Test>());
		});
	}
}*/

