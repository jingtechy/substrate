// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

//! Macro for benchmarking a FRAME runtime.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
mod analysis;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_instance;
mod utils;

pub mod baseline;

#[cfg(feature = "std")]
pub use analysis::{Analysis, AnalysisChoice, BenchmarkSelector};
#[doc(hidden)]
pub use frame_support;
#[doc(hidden)]
pub use log;
#[doc(hidden)]
pub use paste;
#[doc(hidden)]
pub use sp_core::defer;
#[doc(hidden)]
pub use sp_io::storage::root as storage_root;
#[doc(hidden)]
pub use sp_runtime::traits::Zero;
#[doc(hidden)]
pub use sp_runtime::StateVersion;
#[doc(hidden)]
pub use sp_std::{self, boxed::Box, prelude::Vec, str, vec};
pub use sp_storage::{well_known_keys, TrackedStorageKey};
pub use utils::*;

pub mod v1;
pub use v1::*;

/// Contains macros, structs, and traits associated with v2 of the pallet benchmarking syntax.
///
/// The [`v2::benchmarks`] and [`v2::instance_benchmarks`] macros can be used to designate a
/// module as a benchmarking module that can contain benchmarks and benchmark tests. The
/// `#[benchmarks]` variant will set up a regular, non-instance benchmarking module, and the
/// `#[instance_benchmarks]` variant will set up the module in instance benchmarking mode.
///
/// Benchmarking modules should be gated behind a `#[cfg(feature = "runtime-benchmarks")]`
/// feature gate to ensure benchmarking code that is only compiled when the
/// `runtime-benchmarks` feature is enabled is not referenced.
///
/// The following is the general syntax for a benchmarks (or instance benchmarks) module:
///
/// ## General Syntax
///
/// ```ignore
/// #![cfg(feature = "runtime-benchmarks")]
///
/// use super::{mock_helpers::*, Pallet as MyPallet};
/// use frame_benchmarking::v2::*;
///
/// #[benchmarks]
/// mod benchmarks {
/// 	use super::*;
///
/// 	#[benchmark]
/// 	fn bench_name_1(x: Linear<7, 1_000>, y: Linear<1_000, 100_0000>) {
/// 		// setup code
/// 		let z = x + y;
/// 		let caller = whitelisted_caller();
///
/// 		#[extrinsic_call]
/// 		extrinsic_name(SystemOrigin::Signed(caller), other, arguments);
///
/// 		// verification code
/// 		assert_eq!(MyPallet::<T>::my_var(), z);
/// 	}
///
/// 	#[benchmark]
/// 	fn bench_name_2() {
/// 		// setup code
/// 		let caller = whitelisted_caller();
///
/// 		#[block]
/// 		{
/// 			something(some, thing);
/// 			my_extrinsic(RawOrigin::Signed(caller), some, argument);
/// 			something_else(foo, bar);
/// 		}
///
/// 		// verification code
/// 		assert_eq!(MyPallet::<T>::something(), 37);
/// 	}
/// }
/// ```
///
/// ## Benchmark Definitions
///
/// Within a `#[benchmarks]` or `#[instance_benchmarks]` module, you can define individual
/// benchmarks using the `#[benchmark]` attribute, as shown in the example above.
///
/// The `#[benchmark]` attribute expects a function definition with a blank return type and
/// zero or more arguments whose names are valid
/// [BenchmarkParameter](`crate::BenchmarkParameter`) parameters, such as `x`, `y`, `a`, `b`,
/// etc., and whose param types must implement [ParamRange](`v2::ParamRange`). At the moment
/// the only valid type that implements [ParamRange](`v2::ParamRange`) is
/// [Linear](`v2::Linear`).
///
/// The valid syntax for defining a [Linear](`v2::Linear`)is `Linear<A, B>` where `A`, and `B`
/// are valid integer literals (that fit in a `u32`), such that `B` >= `A`.
///
/// Note that the benchmark function definition does not actually expand as a function
/// definition, but rather is used to automatically create a number of impls and structs
/// required by the benchmarking engine. For this reason, the visibility of the function
/// definition as well as the return type are not used for any purpose and are discarded by the
/// expansion code.
///
/// Also note that the `// setup code` and `// verification code` comments shown above are not
/// required and are included simply for demonstration purposes.
///
/// ### `#[extrinsic_call]` and `#[block]`
///
/// Within the benchmark function body, either an `#[extrinsic_call]` or a `#[block]`
/// annotation is required. These attributes should be attached to a block (shown in
/// `bench_name_2` above) or a one-line function call (shown in `bench_name_1` above, in `syn`
/// parlance this should be an `ExprCall`), respectively.
///
/// The `#[block]` syntax is broad and will benchmark any code contained within the block the
/// attribute is attached to. If `#[block]` is attached to something other than a block, a
/// compiler error will be emitted.
///
/// The one-line `#[extrinsic_call]` syntax must consist of a function call to an extrinsic,
/// where the first argument is the origin. If `#[extrinsic_call]` is attached to an item that
/// doesn't meet these requirements, a compiler error will be emitted.
///
/// As a short-hand, you may substitute the name of the extrinsic call with `_`, such as the
/// following:
///
/// ```ignore
/// #[extrinsic_call]
/// _(RawOrigin::Signed(whitelisted_caller()), 0u32.into(), 0);
/// ```
///
/// The underscore will be substituted with the name of the benchmark  (i.e. the name of the
/// function in the benchmark function definition).
///
/// Regardless of whether `#[extrinsic_call]` or `#[block]` is used, this attribute also serves
/// the purpose of designating the boundary between the setup code portion of the benchmark
/// (everything before the `#[extrinsic_call]` or `#[block]` attribute) and the verification
/// stage (everything after the item that the `#[extrinsic_call]` or `#[block]` attribute is
/// attached to). The setup code section should contain any code that needs to execute before
/// the measured portion of the benchmark executes. The verification section is where you can
/// perform assertions to verify that the extrinsic call (or whatever is happening in your
/// block, if you used the `#[block]` syntax) executed successfully.
///
/// Note that neither `#[extrinsic_call]` nor `#[block]` are real attribute macros and are
/// instead consumed by the outer macro pattern as part of the enclosing benchmark function
/// definition. This is why we are able to use `#[extrinsic_call]` and `#[block]` within a
/// function definition even though this behavior has not been stabilized
/// yet—`#[extrinsic_call]` and `#[block]` are parsed and consumed as part of the benchmark
/// definition parsing code, so they never expand as their own attribute macros.
///
/// ### Optional Attributes
///
/// The keywords `extra` and `skip_meta` can be provided as optional arguments to the
/// `#[benchmark]` attribute, i.e. `#[benchmark(extra, skip_meta)]`. Including either of these
/// will enable the `extra` or `skip_meta` option, respectively. These options enable the same
/// behavior they did in the old benchmarking syntax in `frame_benchmarking`, namely:
///
/// #### `extra`
///
/// Specifies that this benchmark should not normally run. To run benchmarks marked with
/// `extra`, you will need to invoke the `frame-benchmarking-cli` with `--extra`.
///
/// #### `skip_meta`
///
/// Specifies that the benchmarking framework should not analyze the storage keys that
/// benchmarked code read or wrote. This useful to suppress the prints in the form of unknown
/// 0x… in case a storage key that does not have metadata. Note that this skips the analysis
/// of all accesses, not just ones without metadata.
///
/// ## Where Clause
///
/// Some pallets require a where clause specifying constraints on their generics to make
/// writing benchmarks feasible. To accomodate this situation, you can provide such a where
/// clause as the (only) argument to the `#[benchmarks]` or `#[instance_benchmarks]` attribute
/// macros. Below is an example of this taken from the `message-queue` pallet.
///
/// ```ignore
/// #[benchmarks(
/// 	where
/// 		<<T as Config>::MessageProcessor as ProcessMessage>::Origin: From<u32> + PartialEq,
/// 		<T as Config>::Size: From<u32>,
/// )]
/// mod benchmarks {
/// 	use super::*;
/// 	// ...
/// }
/// ```
///
/// ## Benchmark Tests
///
/// Benchmark tests can be generated using the old syntax in `frame_benchmarking`,
/// including the `frame_benchmarking::impl_benchmark_test_suite` macro.
///
/// An example is shown below (taken from the `message-queue` pallet's `benchmarking` module):
/// ```ignore
/// #[benchmarks]
/// mod benchmarks {
/// 	use super::*;
/// 	// ...
/// 	impl_benchmark_test_suite!(
/// 		MessageQueue,
/// 		crate::mock::new_test_ext::<crate::integration_test::Test>(),
/// 		crate::integration_test::Test
/// 	);
/// }
/// ```
pub mod v2 {
	pub use super::*;
	pub use frame_support_procedural::{
		benchmark, benchmarks, block, extrinsic_call, instance_benchmarks,
	};

	// Used in #[benchmark] implementation to ensure that benchmark function arguments
	// implement [`ParamRange`].
	#[doc(hidden)]
	pub use static_assertions::assert_impl_all;

	/// Used by the new benchmarking code to specify that a benchmarking variable is linear
	/// over some specified range, i.e. `Linear<0, 1_000>` means that the corresponding variable
	/// is allowed to range from `0` to `1000`, inclusive.
	///
	/// See [`v2`] for more info.
	pub struct Linear<const A: u32, const B: u32>;

	/// Trait that must be implemented by all structs that can be used as parameter range types
	/// in the new benchmarking code (i.e. `Linear<0, 1_000>`). Right now there is just
	/// [`Linear`] but this could later be extended to support additional non-linear parameter
	/// ranges.
	///
	/// See [`v2`] for more info.
	pub trait ParamRange {
		/// Represents the (inclusive) starting number of this `ParamRange`.
		fn start(&self) -> u32;

		/// Represents the (inclusive) ending number of this `ParamRange`.
		fn end(&self) -> u32;
	}

	impl<const A: u32, const B: u32> ParamRange for Linear<A, B> {
		fn start(&self) -> u32 {
			A
		}

		fn end(&self) -> u32 {
			B
		}
	}
}
