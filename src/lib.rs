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

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

//! Confined collections with compile-time size guarantees.
//!
//! The crate is built around a single type, [`Confined`], which wraps a
//! collection and keeps the number of its items within the
//! `MIN_LEN..=MAX_LEN` range. Both limits are const generic parameters, so
//! they are a part of the type:
//!
//! ```
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
//! # Ok::<(), Error>(())
//! ```
//!
//! The limits are checked when a value is constructed and by each method which
//! changes the number of items. [`Confined`] dereferences to the wrapped
//! collection, so all read-only methods of the collection are available, while
//! its size-changing methods are replaced with the checked ones.
//!
//! The crate also provides type aliases for the most common limits, like
//! [`TinyVec`], [`SmallString`] or [`NonEmptyVec`], and macros constructing
//! them, like [`tiny_vec!`] or [`small_bmap!`].

pub extern crate alloc;
extern crate core;

#[cfg(feature = "ascii")]
pub extern crate ascii;
#[cfg(feature = "indexmap")]
pub extern crate indexmap;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

mod error;
mod base;
mod specifics;
mod imp;
mod aliases;
mod macros;

pub use aliases::*;
pub use base::{Confined, Entry, ItemLen};
pub use error::Error;
#[cfg(feature = "ascii")]
pub use imp::AsciiError;

// TODO: Re-name to U*MAX
/// Constant for a minimal size of a confined collection.
pub const ZERO: usize = 0;
/// Constant for a minimal size of a confined collection.
pub const ONE: usize = 1;
/// Constant for a maximal size of a confined collection equal to [`u8::MAX`].
pub const U8MAX: usize = u8::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u16::MAX`].
pub const U16MAX: usize = u16::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u24::MAX`](amplify_num::u24::MAX).
pub const U24MAX: usize = amplify_num::u24::MAX.into_usize();
/// Constant for a maximal size of a confined collection equal to [`u32::MAX`].
pub const U32MAX: usize = u32::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u40::MAX`](amplify_num::u40::MAX).
pub const U40MAX: usize = amplify_num::u40::MAX.into_usize();
/// Constant for a maximal size of a confined collection equal to [`u48::MAX`](amplify_num::u48::MAX).
pub const U48MAX: usize = amplify_num::u48::MAX.into_usize();
/// Constant for a maximal size of a confined collection equal to [`u56::MAX`](amplify_num::u56::MAX).
pub const U56MAX: usize = amplify_num::u56::MAX.into_usize();
/// Constant for a maximal size of a confined collection equal to [`u64::MAX`].
pub const U64MAX: usize = u64::MAX as usize;

/// Runs the examples from `README.md` as documentation tests. The examples
/// need all features of the crate.
#[cfg(all(
    doctest,
    feature = "std",
    feature = "hex",
    feature = "ascii",
    feature = "indexmap",
    feature = "serde"
))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
