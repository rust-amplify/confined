// SPDX-License-Identifier: Apache-2.0
//
// Written in 2022-2026 by Dr. Maxim Orlovsky <orlovsky@ubideco.org>
//
// Copyright 2022-2026 Laboratories for Ubiquitous and Deterministic Computing,
// Institute for Distributed and Cognitive Computing (InDCS), Switzerland.
// All rights reserved.
//
// Copyright (C) 2022-2026 Dr Maxim Orlovsky.
// All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at <http://www.apache.org/licenses/LICENSE-2.0>
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Confined collections with compile-time size guarantees.
//!
//! The crate is built around a single type, [`Confined`], which wraps a
//! collection and keeps the number of its items within the
//! `MIN_LEN..=MAX_LEN` range. Both limits are const generic parameters, so
//! they are a part of the type:
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use confined::{Confined, Error};
//!
//! // From one to three items.
//! let mut signers = Confined::<Vec<&str>, 1, 3>::try_from(vec!["alice"])?;
//! signers.push("bob")?;
//! signers.push("carol")?;
//! assert_eq!(
//!     signers.push("dave"),
//!     Err(Error::Oversize { len: 3, max_len: 3 })
//! );
//! # }
//! # Ok::<(), confined::Error>(())
//! ```
//!
//! The limits are checked when a value is constructed and by each method which
//! changes the number of items. [`Confined`] dereferences to the wrapped
//! collection, so all read-only methods of the collection are available, while
//! its size-changing methods are replaced with the checked ones.
//!
//! With `alloc`, aliases such as `TinyVec`, `SmallString` and `NonEmptyVec`
//! and constructor macros such as `tiny_vec!` and `small_bmap!` are available.
//! Length-limit constants are capped at `usize::MAX` on narrower platforms.

#[cfg(feature = "alloc")]
pub extern crate alloc;

#[cfg(feature = "ascii")]
pub extern crate ascii;
#[cfg(feature = "indexmap")]
pub extern crate indexmap;
#[cfg(feature = "hashbrown")]
pub use collections::hashbrown;
#[cfg(feature = "heapless")]
pub use collections::heapless;

mod error;
mod base;
mod specifics;
mod imp;
#[cfg(feature = "alloc")]
mod aliases;
#[cfg(feature = "heapless")]
mod fixed;
#[cfg(feature = "alloc")]
mod macros;

#[cfg(feature = "alloc")]
pub use aliases::*;
pub use base::{Confined, Entry, ItemLen};
pub use error::Error;
#[cfg(feature = "heapless")]
pub use fixed::*;
#[cfg(feature = "ascii")]
pub use imp::AsciiError;

/// Constant for a minimal size of a confined collection.
pub const ZERO: usize = 0;
/// Constant for a minimal size of a confined collection.
pub const ONE: usize = 1;
/// Constant for a maximal size of a confined collection equal to [`u8::MAX`].
pub const U8MAX: usize = u8::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u16::MAX`].
pub const U16MAX: usize = u16::MAX as usize;
/// Constant for a maximal size of a confined collection equal to
/// [`u24::MAX`](amplify_num::u24::MAX).
pub const U24MAX: usize = max_usize(24);
/// Constant for a maximal size of a confined collection equal to [`u32::MAX`].
pub const U32MAX: usize = max_usize(32);
/// Constant for a maximal size of a confined collection equal to
/// [`u40::MAX`](amplify_num::u40::MAX).
pub const U40MAX: usize = max_usize(40);
/// Constant for a maximal size of a confined collection equal to
/// [`u48::MAX`](amplify_num::u48::MAX).
pub const U48MAX: usize = max_usize(48);
/// Constant for a maximal size of a confined collection equal to
/// [`u56::MAX`](amplify_num::u56::MAX).
pub const U56MAX: usize = max_usize(56);
/// Constant for a maximal size of a confined collection equal to [`u64::MAX`].
pub const U64MAX: usize = max_usize(64);

const fn max_usize(bits: u32) -> usize {
    if usize::BITS <= bits {
        usize::MAX
    } else {
        (1usize << bits) - 1
    }
}

/// Runs the feature-aware examples from `README.md` as documentation tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
