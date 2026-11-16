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

use core::fmt::{self, Display, Formatter};

/// Errors when confinement constraints were not met.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Error {
    /// Operation results in a collection reduced below the required minimum
    /// number of elements.
    Undersize {
        /** Current collection length */
        len: usize,
        /** Minimum number of elements which must be present in the
         * collection */
        min_len: usize,
    },

    /// Operation results in a collection growth above the required maximum
    /// number of elements.
    Oversize {
        /** Current collection length */
        len: usize,
        /** Maximum number of elements which must be present in the
         * collection */
        max_len: usize,
    },

    /// Attempt to address an index outside the collection bounds.
    OutOfBoundary {
        /** Index which was outside the bounds */
        index: usize,
        /** The actual number of elements in the collection */
        len: usize,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Undersize { len, min_len } => write!(
                f,
                "operation results in collection size {len} less than lower boundary of \
                 {min_len}, which is prohibited"
            ),
            Error::Oversize { len, max_len } => write!(
                f,
                "operation results in collection size {len} exceeding {max_len}, which is \
                 prohibited"
            ),
            Error::OutOfBoundary { index, len } => write!(
                f,
                "attempt to access the element at {index} which is outside of the collection \
                 length boundary {len}"
            ),
        }
    }
}

// TODO: Use Error from core with MSRV bump to 1.81.0
#[cfg(feature = "std")]
impl std::error::Error for Error {}
