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

    /// The wrapped collection has no storage capacity for another item.
    CapacityExceeded {
        /** Current collection length */
        len: usize,
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
            Error::CapacityExceeded { len } => write!(
                f,
                "collection capacity cannot hold an item after length {len}"
            ),
            Error::OutOfBoundary { index, len } => write!(
                f,
                "attempt to access the element at {index} which is outside of the collection \
                 length boundary {len}"
            ),
        }
    }
}

impl core::error::Error for Error {}

#[cfg(test)]
mod tests {
    use core::fmt::Write;

    use super::*;

    /// A text buffer which needs no allocator.
    struct Text {
        bytes: [u8; 160],
        len: usize,
    }

    impl Text {
        fn of(error: Error) -> Self {
            let mut text = Text {
                bytes: [0; 160],
                len: 0,
            };
            write!(text, "{error}").expect("the message fits the buffer");
            text
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).expect("the message is UTF-8")
        }
    }

    impl Write for Text {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let end = self.len + s.len();
            self.bytes
                .get_mut(self.len..end)
                .ok_or(fmt::Error)?
                .copy_from_slice(s.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    mod display {
        use super::*;

        #[test]
        fn undersize_names_the_length_and_the_lower_bound() {
            let error = Error::Undersize { len: 1, min_len: 2 };
            assert_eq!(
                Text::of(error).as_str(),
                "operation results in collection size 1 less than lower boundary of 2, which is \
                 prohibited"
            );
        }

        #[test]
        fn oversize_names_the_length_and_the_upper_bound() {
            let error = Error::Oversize { len: 4, max_len: 3 };
            assert_eq!(
                Text::of(error).as_str(),
                "operation results in collection size 4 exceeding 3, which is prohibited"
            );
        }

        #[test]
        fn capacity_exceeded_names_the_length() {
            let error = Error::CapacityExceeded { len: 5 };
            assert_eq!(
                Text::of(error).as_str(),
                "collection capacity cannot hold an item after length 5"
            );
        }

        #[test]
        fn out_of_boundary_names_the_index_and_the_length() {
            let error = Error::OutOfBoundary { index: 7, len: 3 };
            assert_eq!(
                Text::of(error).as_str(),
                "attempt to access the element at 7 which is outside of the collection length \
                 boundary 3"
            );
        }
    }

    mod traits {
        use super::*;

        #[test]
        fn error_is_copied_and_compared_by_value() {
            let error = Error::Oversize { len: 4, max_len: 3 };
            let copy = error;
            assert_eq!(error, copy);
            assert_ne!(error, Error::Oversize { len: 5, max_len: 3 });
            assert_ne!(error, Error::Undersize { len: 4, min_len: 3 });
        }

        #[test]
        fn error_has_no_source() {
            let error = Error::CapacityExceeded { len: 0 };
            assert!(core::error::Error::source(&error).is_none());
        }
    }
}
