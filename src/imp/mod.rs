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

#[cfg(feature = "alloc")]
mod string;
#[cfg(feature = "alloc")]
mod vec;
#[cfg(feature = "alloc")]
mod bytes;
#[cfg(feature = "alloc")]
mod deque;
#[cfg(feature = "alloc")]
mod heap;
#[cfg(feature = "alloc")]
mod list;
#[cfg(feature = "alloc")]
mod btree;
#[cfg(feature = "std")]
mod hashed;
#[cfg(feature = "hashbrown")]
mod hashbrown;
#[cfg(all(feature = "ascii", feature = "alloc"))]
mod ascii;
#[cfg(all(feature = "indexmap", feature = "alloc"))]
mod indexed;

#[cfg(all(feature = "ascii", feature = "alloc"))]
pub use self::ascii::AsciiError;
