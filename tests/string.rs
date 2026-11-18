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

//! Checked methods of the confined `String` (`src/imp/string.rs`).
//!
//! The length of a string is the number of bytes in its UTF-8 encoding, and
//! never the number of its characters. The tests use characters of each
//! encoded length to tell the two apart:
//!
//! | Character | Bytes |
//! |-----------|-------|
//! | `a`       | 1     |
//! | `ñ`       | 2     |
//! | `€`       | 3     |
//! | `𝄞`       | 4     |
//!
//! Each module covers one method, and each test covers one case of it:
//!
//! - construction: `try_from_str`, `from_str`, `try_from_string`,
//!   `try_from_iter`, `with`;
//! - reading: `len`;
//! - growth: `push`, `extend`;
//! - removal: `pop`, `remove`, `clear`.
//!
//! The traits of the confined string are covered by `tests/traits.rs`, and
//! the refused allocations by `tests/oom.rs`.

#![cfg(feature = "alloc")]

use confined::{Confined, Error};

type Text<const MIN: usize, const MAX: usize> = Confined<String, MIN, MAX>;

fn text_of<const MIN: usize, const MAX: usize>(text: &str) -> Text<MIN, MAX> {
    text.parse().unwrap()
}

mod try_from_str {
    use super::*;

    fn convert<const MIN: usize, const MAX: usize>(text: &str) -> Result<Text<MIN, MAX>, Error> {
        <Text<MIN, MAX> as TryFrom<&str>>::try_from(text)
    }

    #[test]
    fn string_within_the_bounds_is_accepted() {
        assert_eq!(convert::<1, 8>("text").unwrap().as_str(), "text");
    }

    #[test]
    fn string_at_the_minimum_is_accepted() {
        assert_eq!(convert::<4, 8>("text").unwrap().as_str(), "text");
    }

    #[test]
    fn string_at_the_maximum_is_accepted() {
        assert_eq!(convert::<1, 4>("text").unwrap().as_str(), "text");
    }

    #[test]
    fn string_below_the_minimum_is_rejected() {
        assert_eq!(
            convert::<5, 8>("text"),
            Err(Error::Undersize { len: 4, min_len: 5 })
        );
    }

    #[test]
    fn string_above_the_maximum_is_rejected() {
        assert_eq!(
            convert::<1, 3>("text"),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
    }

    #[test]
    fn maximum_is_counted_in_bytes_and_not_in_characters() {
        // Three characters in four bytes.
        assert_eq!(
            convert::<0, 3>("añb"),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
        assert_eq!(convert::<0, 4>("añb").unwrap().as_str(), "añb");
    }

    #[test]
    fn minimum_is_counted_in_bytes_and_not_in_characters() {
        // One character in three bytes.
        assert_eq!(convert::<3, 3>("€").unwrap().as_str(), "€");
        assert_eq!(
            convert::<4, 8>("€"),
            Err(Error::Undersize { len: 3, min_len: 4 })
        );
    }

    #[test]
    fn empty_string_fits_a_confinement_without_a_minimum() {
        assert!(convert::<0, 4>("").unwrap().is_empty());
    }

    #[test]
    fn empty_string_is_rejected_by_a_minimum() {
        assert_eq!(
            convert::<1, 4>(""),
            Err(Error::Undersize { len: 0, min_len: 1 })
        );
    }

    #[test]
    fn empty_string_is_the_only_one_which_fits_a_zero_maximum() {
        assert!(convert::<0, 0>("").unwrap().is_empty());
        assert_eq!(
            convert::<0, 0>("a"),
            Err(Error::Oversize { len: 1, max_len: 0 })
        );
    }

    #[test]
    fn string_slice_is_converted_with_try_into() {
        let text: Result<Text<1, 4>, Error> = "text".try_into();
        assert_eq!(text.unwrap().as_str(), "text");
    }

    #[test]
    fn content_is_copied_and_the_slice_is_left_intact() {
        let source = String::from("text");
        let text = convert::<1, 4>(&source).unwrap();
        assert_ne!(text.as_ptr(), source.as_ptr());
        assert_eq!(source, "text");
    }
}

mod from_str {
    use super::*;

    #[test]
    fn string_within_the_bounds_is_parsed() {
        let text: Text<1, 8> = "text".parse().unwrap();
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn string_below_the_minimum_is_not_parsed() {
        assert_eq!(
            "text".parse::<Text<5, 8>>(),
            Err(Error::Undersize { len: 4, min_len: 5 })
        );
    }

    #[test]
    fn string_above_the_maximum_is_not_parsed() {
        assert_eq!(
            "text".parse::<Text<1, 3>>(),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
    }

    #[test]
    fn bytes_are_counted_when_a_string_is_parsed() {
        assert_eq!(
            "€€".parse::<Text<0, 5>>(),
            Err(Error::Oversize { len: 6, max_len: 5 })
        );
        assert_eq!("€€".parse::<Text<6, 6>>().unwrap().len(), 6);
    }

    #[test]
    fn empty_string_is_parsed_without_a_minimum() {
        assert!("".parse::<Text<0, 4>>().unwrap().is_empty());
    }
}

mod try_from_string {
    use super::*;

    #[test]
    fn string_within_the_bounds_is_wrapped() {
        let text = Text::<1, 4>::try_from(String::from("text")).unwrap();
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn string_is_wrapped_without_a_copy() {
        let source = String::from("text");
        let storage = source.as_ptr();
        let text = Text::<1, 4>::try_from(source).unwrap();
        assert_eq!(text.as_ptr(), storage);
    }

    #[test]
    fn string_below_the_minimum_in_bytes_is_rejected() {
        assert_eq!(
            Text::<4, 8>::try_from(String::from("€")),
            Err(Error::Undersize { len: 3, min_len: 4 })
        );
    }

    #[test]
    fn string_above_the_maximum_in_bytes_is_rejected() {
        assert_eq!(
            Text::<0, 3>::try_from(String::from("añb")),
            Err(Error::Oversize { len: 4, max_len: 3 })
        );
    }
}

mod try_from_iter {
    use super::*;

    #[test]
    fn characters_are_collected_in_order() {
        let text = Text::<1, 8>::try_from_iter("a€b".chars()).unwrap();
        assert_eq!(text.as_str(), "a€b");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn character_which_does_not_fit_is_reported_with_the_length_it_makes() {
        // Four bytes are collected, and the next character takes three more.
        assert_eq!(
            Text::<0, 5>::try_from_iter("a€€".chars()),
            Err(Error::Oversize { len: 7, max_len: 5 })
        );
    }

    #[test]
    fn too_few_bytes_are_reported_when_the_iterator_ends() {
        assert_eq!(
            Text::<4, 8>::try_from_iter(['€']),
            Err(Error::Undersize { len: 3, min_len: 4 })
        );
    }

    #[test]
    fn empty_iterator_makes_an_empty_string() {
        assert!(Text::<0, 4>::try_from_iter([]).unwrap().is_empty());
    }
}

mod with {
    use super::*;

    #[test]
    fn one_byte_character_makes_a_string_of_one_byte() {
        let text = Text::<1, 4>::with('a');
        assert_eq!(text.as_str(), "a");
        assert_eq!(text.len(), 1);
    }

    #[test]
    fn largest_character_fits_the_least_possible_maximum() {
        let text = Text::<1, 4>::with('𝄞');
        assert_eq!(text.as_str(), "𝄞");
        assert_eq!(text.len(), 4);
    }

    #[test]
    fn single_character_fits_a_confinement_without_a_minimum() {
        assert_eq!(Text::<0, 8>::with('€').as_str(), "€");
    }
}

mod len {
    use super::*;

    #[test]
    fn length_is_the_number_of_bytes() {
        let text = text_of::<0, 16>("añ€𝄞");
        assert_eq!(text.len(), 1 + 2 + 3 + 4);
        assert_eq!(text.chars().count(), 4);
    }

    #[test]
    fn length_of_an_ascii_string_is_the_number_of_its_characters() {
        assert_eq!(text_of::<0, 16>("text").len(), 4);
    }

    #[test]
    fn empty_string_has_no_length() {
        let text = Text::<0, 4>::new();
        assert_eq!(text.len(), 0);
        assert!(text.is_empty());
    }

    #[test]
    fn length_fits_the_integer_of_the_maximum() {
        assert_eq!(text_of::<0, 255>("añ").len_u8(), 3);
    }
}

mod push {
    use super::*;

    #[test]
    fn one_byte_character_adds_one_byte() {
        let mut text = text_of::<0, 5>("ab");
        assert_eq!(text.push('c'), Ok(()));
        assert_eq!(text.as_str(), "abc");
        assert_eq!(text.len(), 3);
    }

    #[test]
    fn character_of_several_bytes_fills_the_string_exactly() {
        let mut text = text_of::<0, 5>("ab");
        assert_eq!(text.push('€'), Ok(()));
        assert_eq!(text.as_str(), "ab€");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn character_is_refused_when_its_bytes_do_not_fit_the_room_left() {
        // Two bytes are left, and the character takes three.
        let mut text = text_of::<0, 5>("abc");
        assert_eq!(text.push('€'), Err(Error::Oversize { len: 3, max_len: 5 }));
        assert_eq!(text.as_str(), "abc");
    }

    #[test]
    fn shorter_character_is_accepted_after_a_refused_one() {
        let mut text = text_of::<0, 5>("abc");
        assert!(text.push('€').is_err());
        assert_eq!(text.push('ñ'), Ok(()));
        assert_eq!(text.as_str(), "abcñ");
        assert_eq!(text.len(), 5);
    }

    #[test]
    fn character_is_refused_at_the_maximum() {
        let mut text = text_of::<0, 4>("text");
        assert_eq!(text.push('s'), Err(Error::Oversize { len: 4, max_len: 4 }));
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn largest_character_needs_four_bytes() {
        let mut text = text_of::<0, 8>("abcde");
        assert_eq!(text.push('𝄞'), Err(Error::Oversize { len: 5, max_len: 8 }));
        let mut text = text_of::<0, 8>("abcd");
        assert_eq!(text.push('𝄞'), Ok(()));
        assert_eq!(text.len(), 8);
    }

    #[test]
    fn character_is_refused_by_a_zero_maximum() {
        let mut text = Text::<0, 0>::new();
        assert_eq!(text.push('a'), Err(Error::Oversize { len: 0, max_len: 0 }));
        assert!(text.is_empty());
    }
}

mod extend {
    use super::*;

    #[test]
    fn fitting_characters_are_added() {
        let mut text = text_of::<0, 8>("a");
        assert_eq!(text.extend("ñ€".chars()), Ok(()));
        assert_eq!(text.as_str(), "añ€");
        assert_eq!(text.len(), 6);
    }

    #[test]
    fn characters_before_the_one_which_does_not_fit_are_kept() {
        let mut text = text_of::<0, 5>("a");
        assert_eq!(
            text.extend("b€cd".chars()),
            Err(Error::Oversize { len: 5, max_len: 5 })
        );
        assert_eq!(text.as_str(), "ab€");
    }

    #[test]
    fn extension_stops_at_a_character_which_does_not_fit_the_room_left() {
        // The second character would fit, but it is not reached.
        let mut text = text_of::<0, 5>("abc");
        assert_eq!(
            text.extend(['€', 'd']),
            Err(Error::Oversize { len: 3, max_len: 5 })
        );
        assert_eq!(text.as_str(), "abc");
    }

    #[test]
    fn empty_iterator_adds_nothing_at_the_maximum() {
        let mut text = text_of::<0, 4>("text");
        assert_eq!(text.extend([]), Ok(()));
        assert_eq!(text.as_str(), "text");
    }
}

mod pop {
    use super::*;

    #[test]
    fn last_character_is_returned() {
        let mut text = text_of::<0, 8>("text");
        assert_eq!(text.pop(), Some('t'));
        assert_eq!(text.as_str(), "tex");
    }

    #[test]
    fn character_of_several_bytes_is_returned_whole() {
        let mut text = text_of::<0, 8>("a€");
        assert_eq!(text.pop(), Some('€'));
        assert_eq!(text.as_str(), "a");
        assert_eq!(text.len(), 1);
    }

    #[test]
    fn characters_are_returned_from_the_end_until_the_string_is_empty() {
        let mut text = text_of::<0, 8>("ñ𝄞");
        assert_eq!(text.pop(), Some('𝄞'));
        assert_eq!(text.pop(), Some('ñ'));
        assert_eq!(text.pop(), None);
        assert!(text.is_empty());
    }

    #[test]
    fn empty_string_returns_nothing() {
        let mut text = Text::<0, 8>::new();
        assert_eq!(text.pop(), None);
        assert!(text.is_empty());
    }

    #[test]
    fn full_string_accepts_a_character_after_a_pop() {
        let mut text = text_of::<0, 4>("a€");
        assert_eq!(text.pop(), Some('€'));
        assert_eq!(text.extend("bcd".chars()), Ok(()));
        assert_eq!(text.as_str(), "abcd");
    }
}

mod remove {
    use super::*;

    #[test]
    fn character_at_the_index_is_returned_and_the_rest_is_shifted() {
        let mut text = text_of::<1, 8>("text");
        assert_eq!(text.remove(1), Ok('e'));
        assert_eq!(text.as_str(), "txt");
    }

    #[test]
    fn first_character_is_removed() {
        let mut text = text_of::<1, 8>("text");
        assert_eq!(text.remove(0), Ok('t'));
        assert_eq!(text.as_str(), "ext");
    }

    #[test]
    fn last_character_is_removed() {
        let mut text = text_of::<1, 8>("text");
        assert_eq!(text.remove(3), Ok('t'));
        assert_eq!(text.as_str(), "tex");
    }

    #[test]
    fn index_is_a_position_in_bytes() {
        // The character `b` is the third one, and it starts at the byte 4.
        let mut text = text_of::<1, 8>("a€b");
        assert_eq!(text.remove(4), Ok('b'));
        assert_eq!(text.as_str(), "a€");
    }

    #[test]
    fn character_of_several_bytes_is_removed_by_the_index_of_its_first_byte() {
        let mut text = text_of::<1, 8>("a€b");
        assert_eq!(text.remove(1), Ok('€'));
        assert_eq!(text.as_str(), "ab");
        assert_eq!(text.len(), 2);
    }

    #[test]
    fn index_inside_a_character_is_out_of_boundary() {
        // The bytes 2 and 3 are in the middle of `€`. The wrapped string
        // panics for such an index.
        let mut text = text_of::<1, 8>("a€b");
        for index in [2, 3] {
            assert_eq!(
                text.remove(index),
                Err(Error::OutOfBoundary { index, len: 5 })
            );
        }
        assert_eq!(text.as_str(), "a€b");
    }

    #[test]
    fn index_equal_to_the_length_is_out_of_boundary() {
        let mut text = text_of::<1, 8>("a€b");
        assert_eq!(
            text.remove(5),
            Err(Error::OutOfBoundary { index: 5, len: 5 })
        );
        assert_eq!(text.as_str(), "a€b");
    }

    #[test]
    fn index_of_the_last_character_counted_in_characters_may_be_a_wrong_one() {
        // The string has three characters, but the index 2 is not the third.
        let mut text = text_of::<1, 8>("a€b");
        assert_eq!(
            text.remove(2),
            Err(Error::OutOfBoundary { index: 2, len: 5 })
        );
    }

    #[test]
    fn largest_index_is_out_of_boundary() {
        let mut text = text_of::<1, 8>("text");
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
        let mut text = text_of::<4, 8>("text");
        assert_eq!(text.remove(0), Err(Error::Undersize { len: 4, min_len: 4 }));
        assert_eq!(text.as_str(), "text");
    }

    #[test]
    fn character_is_kept_when_its_bytes_cross_the_minimum() {
        // Four bytes with the minimum of two: a character of one byte can
        // be removed, and a character of three bytes can not.
        let mut text = text_of::<2, 8>("a€");
        assert_eq!(text.remove(1), Err(Error::Undersize { len: 4, min_len: 2 }));
        assert_eq!(text.as_str(), "a€");
        assert_eq!(text.remove(0), Ok('a'));
        assert_eq!(text.as_str(), "€");
    }

    #[test]
    fn character_is_removed_down_to_the_minimum_exactly() {
        let mut text = text_of::<1, 8>("a€");
        assert_eq!(text.remove(1), Ok('€'));
        assert_eq!(text.as_str(), "a");
        assert_eq!(text.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));
    }

    #[test]
    fn index_is_checked_before_the_minimum() {
        let mut text = text_of::<5, 8>("a€b");
        assert_eq!(
            text.remove(2),
            Err(Error::OutOfBoundary { index: 2, len: 5 })
        );
        assert_eq!(
            text.remove(9),
            Err(Error::OutOfBoundary { index: 9, len: 5 })
        );
        assert_eq!(text.remove(0), Err(Error::Undersize { len: 5, min_len: 5 }));
        assert_eq!(text.as_str(), "a€b");
    }

    #[test]
    fn only_character_is_removed_without_a_minimum() {
        let mut text = text_of::<0, 8>("𝄞");
        assert_eq!(text.remove(0), Ok('𝄞'));
        assert!(text.is_empty());
    }

    #[test]
    fn empty_string_has_no_character_to_remove() {
        let mut text = Text::<0, 8>::new();
        assert_eq!(
            text.remove(0),
            Err(Error::OutOfBoundary { index: 0, len: 0 })
        );
        assert!(text.is_empty());
    }
}

mod clear {
    use super::*;

    #[test]
    fn all_characters_are_removed() {
        let mut text = text_of::<0, 8>("a€b");
        text.clear();
        assert!(text.is_empty());
        assert_eq!(text.as_str(), "");
    }

    #[test]
    fn cleared_string_is_filled_up_to_the_maximum_again() {
        let mut text = text_of::<0, 3>("abc");
        text.clear();
        assert_eq!(text.push('€'), Ok(()));
        assert_eq!(text.push('a'), Err(Error::Oversize { len: 3, max_len: 3 }));
        assert_eq!(text.as_str(), "€");
    }
}
