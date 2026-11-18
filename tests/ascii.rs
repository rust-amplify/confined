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

//! Checked methods of the confined `AsciiString` and the error of its
//! construction (`src/imp/ascii.rs`).
//!
//! Each module covers one item, and each test covers one case of it:
//!
//! - `error`: `AsciiError`;
//! - construction: `try_from_str`, `from_str`, `try_from_string`,
//!   `try_from_iter`, `with`;
//! - growth: `push`, `extend`;
//! - removal: `pop`, `remove`, `clear`;
//! - reading: `first_last`, `display`.
//!
//! A string slice is checked in two steps: its length in bytes first, and its
//! characters after that. The error tells which of the steps has failed.

#![cfg(feature = "ascii")]

use confined::ascii::{AsAsciiStr, AsAsciiStrError, AsciiChar, AsciiString};
use confined::{AsciiError, Confined, Error};

type Ascii<const MIN: usize, const MAX: usize> = Confined<AsciiString, MIN, MAX>;

fn ascii_of<const MIN: usize, const MAX: usize>(text: &str) -> Ascii<MIN, MAX> {
    text.parse().unwrap()
}

/// The error which the `ascii` crate reports for the text, with the position
/// of its first byte which is not ASCII.
fn not_ascii(text: &str) -> AsAsciiStrError {
    text.as_ascii_str().unwrap_err()
}

mod error {
    use super::*;

    const CONFINEMENT: Error = Error::Oversize { len: 4, max_len: 3 };

    #[test]
    fn character_error_is_displayed_as_its_source() {
        let source = not_ascii("añ");
        assert_eq!(AsciiError::Ascii(source).to_string(), source.to_string());
    }

    #[test]
    fn confinement_error_is_displayed_as_its_source() {
        assert_eq!(
            AsciiError::Confinement(CONFINEMENT).to_string(),
            CONFINEMENT.to_string()
        );
    }

    #[test]
    fn character_error_is_converted_from_its_source() {
        let source = not_ascii("añ");
        assert_eq!(AsciiError::from(source), AsciiError::Ascii(source));
    }

    #[test]
    fn confinement_error_is_converted_from_its_source() {
        assert_eq!(
            AsciiError::from(CONFINEMENT),
            AsciiError::Confinement(CONFINEMENT)
        );
    }

    #[test]
    fn error_is_an_error_without_a_source() {
        let error: &dyn core::error::Error = &AsciiError::Confinement(CONFINEMENT);
        assert!(error.source().is_none());
        let error: &dyn core::error::Error = &AsciiError::Ascii(not_ascii("ñ"));
        assert!(error.source().is_none());
    }

    #[test]
    fn errors_are_copied_and_compared_by_value() {
        let error = AsciiError::Confinement(CONFINEMENT);
        let copy = error;
        assert_eq!(error, copy);
        assert_ne!(error, AsciiError::Ascii(not_ascii("ñ")));
        assert_ne!(
            AsciiError::Ascii(not_ascii("ñ")),
            AsciiError::Ascii(not_ascii("añ"))
        );
    }

    #[test]
    fn error_is_printed_for_debugging() {
        let text = format!("{:?}", AsciiError::Confinement(CONFINEMENT));
        assert_eq!(text, "Confinement(Oversize { len: 4, max_len: 3 })");
    }
}

mod try_from_str {
    use super::*;

    fn convert<const MIN: usize, const MAX: usize>(
        text: &str,
    ) -> Result<Ascii<MIN, MAX>, AsciiError> {
        <Ascii<MIN, MAX> as TryFrom<&str>>::try_from(text)
    }

    #[test]
    fn ascii_string_within_the_bounds_is_accepted() {
        assert_eq!(convert::<1, 8>("text").unwrap().as_str(), "text");
    }

    #[test]
    fn ascii_string_at_the_minimum_is_accepted() {
        assert_eq!(convert::<4, 8>("text").unwrap().as_str(), "text");
    }

    #[test]
    fn ascii_string_at_the_maximum_is_accepted() {
        assert_eq!(convert::<1, 4>("text").unwrap().as_str(), "text");
    }

    #[test]
    fn string_below_the_minimum_is_a_confinement_error() {
        assert_eq!(
            convert::<5, 8>("text"),
            Err(AsciiError::Confinement(Error::Undersize {
                len: 4,
                min_len: 5
            }))
        );
    }

    #[test]
    fn string_above_the_maximum_is_a_confinement_error() {
        assert_eq!(
            convert::<1, 3>("text"),
            Err(AsciiError::Confinement(Error::Oversize {
                len: 4,
                max_len: 3
            }))
        );
    }

    #[test]
    fn other_characters_are_a_character_error() {
        assert_eq!(
            convert::<1, 8>("añb"),
            Err(AsciiError::Ascii(not_ascii("añb")))
        );
        assert_eq!(not_ascii("añb").valid_up_to(), 1);
    }

    #[test]
    fn length_is_checked_before_the_characters() {
        assert_eq!(
            convert::<1, 4>("ñññ"),
            Err(AsciiError::Confinement(Error::Oversize {
                len: 6,
                max_len: 4
            }))
        );
    }

    #[test]
    fn length_of_a_rejected_string_is_counted_in_bytes() {
        // A single character in two bytes: it exceeds the maximum of one,
        // and reaches the characters check with the maximum of two.
        assert_eq!(
            convert::<0, 1>("ñ"),
            Err(AsciiError::Confinement(Error::Oversize {
                len: 2,
                max_len: 1
            }))
        );
        assert_eq!(convert::<2, 2>("ñ"), Err(AsciiError::Ascii(not_ascii("ñ"))));
    }

    #[test]
    fn empty_string_fits_a_confinement_without_a_minimum() {
        assert!(convert::<0, 4>("").unwrap().is_empty());
    }

    #[test]
    fn empty_string_is_rejected_by_a_minimum() {
        assert_eq!(
            convert::<1, 4>(""),
            Err(AsciiError::Confinement(Error::Undersize {
                len: 0,
                min_len: 1
            }))
        );
    }

    #[test]
    fn string_slice_is_converted_with_try_into() {
        let text: Result<Ascii<1, 4>, AsciiError> = "text".try_into();
        assert_eq!(text.unwrap().as_str(), "text");
    }
}

mod from_str {
    use super::*;

    #[test]
    fn ascii_string_within_the_bounds_is_parsed() {
        let text: Ascii<1, 8> = "text".parse().unwrap();
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn string_out_of_the_bounds_is_not_parsed() {
        assert_eq!(
            "text".parse::<Ascii<5, 8>>(),
            Err(AsciiError::Confinement(Error::Undersize {
                len: 4,
                min_len: 5
            }))
        );
        assert_eq!(
            "text".parse::<Ascii<1, 3>>(),
            Err(AsciiError::Confinement(Error::Oversize {
                len: 4,
                max_len: 3
            }))
        );
    }

    #[test]
    fn other_characters_are_not_parsed() {
        assert_eq!(
            "te€t".parse::<Ascii<1, 8>>(),
            Err(AsciiError::Ascii(not_ascii("te€t")))
        );
    }
}

mod try_from_string {
    use super::*;

    fn string(text: &str) -> AsciiString {
        AsciiString::from_ascii(text).unwrap()
    }

    #[test]
    fn string_within_the_bounds_is_wrapped() {
        let text = Ascii::<1, 4>::try_from(string("text")).unwrap();
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn string_below_the_minimum_is_rejected() {
        assert_eq!(
            Ascii::<5, 8>::try_from(string("text")),
            Err(Error::Undersize { len: 4, min_len: 5 })
        );
    }

    #[test]
    fn string_above_the_maximum_is_rejected() {
        assert_eq!(
            Ascii::<1, 3>::try_from(string("text")),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
    }
}

mod try_from_iter {
    use super::*;

    #[test]
    fn characters_are_collected_in_order() {
        let text = Ascii::<1, 3>::try_from_iter([AsciiChar::b, AsciiChar::a]).unwrap();
        assert_eq!(text.as_str(), "ba");
    }

    #[test]
    fn character_above_the_maximum_is_reported_with_the_length_it_makes() {
        assert_eq!(
            Ascii::<0, 2>::try_from_iter([AsciiChar::a; 5]),
            Err(Error::Oversize { len: 3, max_len: 2 })
        );
    }

    #[test]
    fn too_few_characters_are_reported_when_the_iterator_ends() {
        assert_eq!(
            Ascii::<2, 3>::try_from_iter([AsciiChar::a]),
            Err(Error::Undersize { len: 1, min_len: 2 })
        );
    }
}

mod with {
    use super::*;

    #[test]
    fn single_character_makes_a_string() {
        let text = Ascii::<1, 1>::with(AsciiChar::a);
        assert_eq!(text.as_str(), "a");
        assert_eq!(text.len(), 1);
    }
}

mod push {
    use super::*;

    #[test]
    fn character_is_added_below_the_maximum() {
        let mut text = ascii_of::<0, 3>("ab");
        assert_eq!(text.push(AsciiChar::c), Ok(()));
        assert_eq!(text.as_str(), "abc");
    }

    #[test]
    fn character_is_refused_at_the_maximum() {
        let mut text = ascii_of::<0, 2>("ab");
        assert_eq!(
            text.push(AsciiChar::c),
            Err(Error::Oversize { len: 2, max_len: 2 })
        );
        assert_eq!(text.as_str(), "ab");
    }

    #[test]
    fn character_is_refused_by_a_zero_maximum() {
        let mut text = Ascii::<0, 0>::new();
        assert_eq!(
            text.push(AsciiChar::a),
            Err(Error::Oversize { len: 0, max_len: 0 })
        );
        assert!(text.is_empty());
    }
}

mod extend {
    use super::*;

    #[test]
    fn fitting_characters_are_added() {
        let mut text = ascii_of::<0, 3>("a");
        assert_eq!(text.extend([AsciiChar::b, AsciiChar::c]), Ok(()));
        assert_eq!(text.as_str(), "abc");
    }

    #[test]
    fn characters_before_the_one_which_does_not_fit_are_kept() {
        let mut text = ascii_of::<0, 3>("a");
        assert_eq!(
            text.extend([AsciiChar::b, AsciiChar::c, AsciiChar::d]),
            Err(Error::Oversize { len: 3, max_len: 3 })
        );
        assert_eq!(text.as_str(), "abc");
    }
}

mod pop {
    use super::*;

    #[test]
    fn last_character_is_returned() {
        let mut text = ascii_of::<0, 3>("ab");
        assert_eq!(text.pop(), Some(AsciiChar::b));
        assert_eq!(text.as_str(), "a");
    }

    #[test]
    fn characters_are_returned_from_the_end_until_the_string_is_empty() {
        let mut text = ascii_of::<0, 3>("ab");
        assert_eq!(text.pop(), Some(AsciiChar::b));
        assert_eq!(text.pop(), Some(AsciiChar::a));
        assert_eq!(text.pop(), None);
        assert!(text.is_empty());
    }

    #[test]
    fn empty_string_returns_nothing() {
        let mut text = Ascii::<0, 3>::new();
        assert_eq!(text.pop(), None);
        assert!(text.is_empty());
    }
}

mod remove {
    use super::*;

    #[test]
    fn character_at_the_index_is_returned_and_the_rest_is_shifted() {
        let mut text = ascii_of::<1, 4>("text");
        assert_eq!(text.remove(1), Ok(AsciiChar::e));
        assert_eq!(text.as_str(), "txt");
    }

    #[test]
    fn first_and_last_characters_are_removed() {
        let mut text = ascii_of::<1, 4>("text");
        assert_eq!(text.remove(0), Ok(AsciiChar::t));
        assert_eq!(text.remove(2), Ok(AsciiChar::t));
        assert_eq!(text.as_str(), "ex");
    }

    #[test]
    fn index_equal_to_the_length_is_out_of_boundary() {
        let mut text = ascii_of::<1, 4>("text");
        assert_eq!(
            text.remove(4),
            Err(Error::OutOfBoundary { index: 4, len: 4 })
        );
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn largest_index_is_out_of_boundary() {
        let mut text = ascii_of::<1, 4>("text");
        assert_eq!(
            text.remove(usize::MAX),
            Err(Error::OutOfBoundary {
                index: usize::MAX,
                len: 4
            })
        );
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn character_is_kept_at_the_minimum() {
        let mut text = ascii_of::<4, 8>("text");
        assert_eq!(text.remove(0), Err(Error::Undersize { len: 4, min_len: 4 }));
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn index_is_checked_before_the_minimum() {
        let mut text = ascii_of::<4, 8>("text");
        assert_eq!(
            text.remove(4),
            Err(Error::OutOfBoundary { index: 4, len: 4 })
        );
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn only_character_is_removed_without_a_minimum() {
        let mut text = ascii_of::<0, 4>("a");
        assert_eq!(text.remove(0), Ok(AsciiChar::a));
        assert!(text.is_empty());
    }

    #[test]
    fn empty_string_has_no_character_to_remove() {
        let mut text = Ascii::<0, 4>::new();
        assert_eq!(
            text.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        );
    }
}

mod clear {
    use super::*;

    #[test]
    fn all_characters_are_removed() {
        let mut text = ascii_of::<0, 4>("text");
        text.clear();
        assert!(text.is_empty());
    }
}

mod first_last {
    use super::*;

    #[test]
    fn ends_of_a_string_are_returned() {
        let text = ascii_of::<1, 4>("abc");
        assert_eq!(*text.first(), AsciiChar::a);
        assert_eq!(*text.last(), AsciiChar::c);
    }

    #[test]
    fn single_character_is_at_both_ends() {
        let text = ascii_of::<1, 4>("a");
        assert_eq!(*text.first(), AsciiChar::a);
        assert_eq!(*text.last(), AsciiChar::a);
    }

    #[test]
    fn ends_are_changed_in_place() {
        let mut text = ascii_of::<1, 4>("abc");
        *text.first_mut() = AsciiChar::x;
        *text.last_mut() = AsciiChar::z;
        assert_eq!(text.as_str(), "xbz");
        assert_eq!(text.len(), 3);
    }
}

mod display {
    use super::*;

    #[test]
    fn string_is_displayed_as_its_content() {
        assert_eq!(ascii_of::<1, 4>("text").to_string(), "text");
    }

    #[test]
    fn length_is_the_number_of_characters() {
        let text = ascii_of::<0, 255>("text");
        assert_eq!(text.len(), 4);
        assert_eq!(text.len_u8(), 4);
    }
}
