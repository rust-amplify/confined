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

pub use base::Confined;
pub use error::Error;

// TODO: Re-name to U*MAX
/// Constant for a minimal size of a confined collection.
pub const ZERO: usize = 0;
/// Constant for a minimal size of a confined collection.
pub const ONE: usize = 1;
/// Constant for a maximal size of a confined collection equal to [`u8::MAX`].
pub const U8: usize = u8::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u16::MAX`].
pub const U16: usize = u16::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u24::MAX`].
pub const U24: usize = 0xFFFFFFusize;
/// Constant for a maximal size of a confined collection equal to [`u32::MAX`].
pub const U32: usize = u32::MAX as usize;
/// Constant for a maximal size of a confined collection equal to [`u64::MAX`].
pub const U64: usize = u64::MAX as usize;
