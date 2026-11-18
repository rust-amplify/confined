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

//! Byte vectors as write buffers and as hex strings (`src/imp/bytes.rs`).
//!
//! Each module covers one trait implemented by the confined `Vec<u8>`, and
//! each test covers one case of it:
//!
//! - `write`: `std::io::Write`, with the `std` feature;
//! - `hex_format`: `LowerHex` and `UpperHex`, with the `hex` feature;
//! - `hex_parse`: `FromHex`, with the `hex` feature.
//!
//! The lengths reported by a failed hex decoding are counted in bytes, and
//! not in the characters of the hex string.
//!
//! The decoding of the iterators which misreport their length is covered by
//! `tests/audit.rs`, the vectors with a custom allocator by
//! `tests/allocator.rs`, and the refused allocations by `tests/oom.rs`.

#![cfg(feature = "alloc")]

#[cfg(any(feature = "std", feature = "hex"))]
use confined::Confined;

#[cfg(any(feature = "std", feature = "hex"))]
type Blob<const MIN: usize, const MAX: usize> = Confined<Vec<u8>, MIN, MAX>;

#[cfg(any(feature = "std", feature = "hex"))]
fn blob_of<const MIN: usize, const MAX: usize>(bytes: &[u8]) -> Blob<MIN, MAX> {
    Blob::try_from(bytes.to_vec()).unwrap()
}

#[cfg(feature = "std")]
mod write {
    use std::io::{ErrorKind, Write};

    use super::*;

    #[test]
    fn buffer_is_appended_and_its_length_is_returned() {
        let mut blob = blob_of::<0, 8>(&[1, 2]);
        assert_eq!(blob.write(&[3, 4, 5]).unwrap(), 3);
        assert_eq!(blob.as_slice(), [1, 2, 3, 4, 5]);
    }

    #[test]
    fn buffer_which_fills_the_room_left_exactly_is_written() {
        let mut blob = blob_of::<0, 4>(&[1, 2]);
        assert_eq!(blob.write(&[3, 4]).unwrap(), 2);
        assert_eq!(blob.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn buffer_larger_than_the_room_left_is_refused_as_a_whole() {
        let mut blob = blob_of::<0, 4>(&[1, 2]);
        let error = blob.write(&[3, 4, 5]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::OutOfMemory);
        assert_eq!(blob.as_slice(), [1, 2]);
    }

    #[test]
    fn buffer_is_refused_at_the_maximum() {
        let mut blob = blob_of::<0, 2>(&[1, 2]);
        let error = blob.write(&[3]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::OutOfMemory);
        assert_eq!(blob.as_slice(), [1, 2]);
    }

    #[test]
    fn empty_buffer_writes_nothing() {
        let mut blob = blob_of::<0, 4>(&[1, 2]);
        assert_eq!(blob.write(&[]).unwrap(), 0);
        assert_eq!(blob.as_slice(), [1, 2]);
    }

    #[test]
    fn empty_buffer_is_accepted_at_the_maximum() {
        let mut blob = blob_of::<0, 2>(&[1, 2]);
        assert_eq!(blob.write(&[]).unwrap(), 0);
        assert_eq!(blob.as_slice(), [1, 2]);
    }

    #[test]
    fn smaller_buffer_is_written_after_a_refused_one() {
        let mut blob = blob_of::<0, 4>(&[1, 2]);
        assert!(blob.write(&[3, 4, 5]).is_err());
        assert_eq!(blob.write(&[3]).unwrap(), 1);
        assert_eq!(blob.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn whole_buffer_is_written_by_write_all() {
        let mut blob = Blob::<0, 4>::new();
        blob.write_all(&[1, 2, 3, 4]).unwrap();
        assert_eq!(blob.as_slice(), [1, 2, 3, 4]);
    }

    #[test]
    fn nothing_is_written_by_a_refused_write_all() {
        let mut blob = blob_of::<0, 4>(&[1, 2]);
        let error = blob.write_all(&[3, 4, 5]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::OutOfMemory);
        assert_eq!(blob.as_slice(), [1, 2]);
    }

    #[test]
    fn formatted_text_is_written() {
        let mut blob = Blob::<0, 8>::new();
        let (number, word) = (12, "ab");
        write!(blob, "{number}-{word}").unwrap();
        assert_eq!(blob.as_slice(), b"12-ab");
    }

    #[test]
    fn formatted_text_which_does_not_fit_is_an_error() {
        let mut blob = Blob::<0, 4>::new();
        assert!(write!(blob, "{}", 12_345).is_err());
        assert!(blob.len() <= 4);
    }

    #[test]
    fn flush_changes_nothing() {
        let mut blob = blob_of::<0, 4>(&[1, 2]);
        blob.flush().unwrap();
        assert_eq!(blob.as_slice(), [1, 2]);
    }

    #[test]
    fn vector_with_a_minimum_is_written_to() {
        let mut blob = blob_of::<2, 4>(&[1, 2]);
        assert_eq!(blob.write(&[3]).unwrap(), 1);
        assert!(blob.write(&[4, 5]).is_err());
        assert_eq!(blob.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn vector_is_written_to_through_a_generic_writer() {
        fn emit(writer: &mut impl Write) -> std::io::Result<()> {
            writer.write_all(&[0xCA, 0xFE])
        }

        let mut blob = Blob::<0, 2>::new();
        emit(&mut blob).unwrap();
        assert_eq!(emit(&mut blob).unwrap_err().kind(), ErrorKind::OutOfMemory);
        assert_eq!(blob.as_slice(), [0xCA, 0xFE]);
    }
}

#[cfg(feature = "hex")]
mod hex_format {
    use super::*;

    #[test]
    fn bytes_are_printed_in_the_lower_case() {
        let blob = blob_of::<0, 4>(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(format!("{blob:x}"), "deadbeef");
    }

    #[test]
    fn bytes_are_printed_in_the_upper_case() {
        let blob = blob_of::<0, 4>(&[0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(format!("{blob:X}"), "DEADBEEF");
    }

    #[test]
    fn each_byte_takes_two_digits() {
        let blob = blob_of::<0, 4>(&[0x00, 0x0A, 0xA0, 0x01]);
        assert_eq!(format!("{blob:x}"), "000aa001");
        assert_eq!(format!("{blob:X}"), "000AA001");
    }

    #[test]
    fn empty_vector_is_an_empty_string() {
        let blob = Blob::<0, 4>::new();
        assert_eq!(format!("{blob:x}"), "");
        assert_eq!(format!("{blob:X}"), "");
        assert_eq!(format!("{blob:#x}"), "");
    }

    #[test]
    fn every_byte_is_printed_like_a_number_of_two_digits() {
        let bytes = (0..=u8::MAX).collect::<Vec<_>>();
        let expected_lower = bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let expected_upper = bytes
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<String>();
        let blob = Blob::<0, 256>::try_from(bytes).unwrap();
        assert_eq!(format!("{blob:x}"), expected_lower);
        assert_eq!(format!("{blob:X}"), expected_upper);
    }

    #[test]
    fn alternate_form_prints_sixteen_bytes_in_full() {
        let blob = Blob::<0, 32>::try_from_iter(0x10..0x20).unwrap();
        assert_eq!(format!("{blob:#x}"), "101112131415161718191a1b1c1d1e1f");
        assert_eq!(format!("{blob:#X}"), "101112131415161718191A1B1C1D1E1F");
    }

    #[test]
    fn alternate_form_abbreviates_seventeen_bytes() {
        // The first eight bytes, two dots, and the last eight bytes: the byte
        // in the middle is left out.
        let blob = Blob::<0, 32>::try_from_iter(0x10..0x21).unwrap();
        assert_eq!(format!("{blob:#x}"), "1011121314151617..191a1b1c1d1e1f20");
        assert_eq!(format!("{blob:#X}"), "1011121314151617..191A1B1C1D1E1F20");
    }

    #[test]
    fn alternate_form_abbreviates_a_long_vector_to_its_ends() {
        let blob = Blob::<0, 256>::try_from_iter(0..=u8::MAX).unwrap();
        assert_eq!(format!("{blob:#x}"), "0001020304050607..f8f9fafbfcfdfeff");
    }

    #[test]
    fn plain_form_prints_a_long_vector_in_full() {
        let blob = Blob::<0, 32>::try_from_iter(0x10..0x21).unwrap();
        assert_eq!(format!("{blob:x}"), "101112131415161718191a1b1c1d1e1f20");
    }

    #[test]
    fn alternate_form_adds_no_prefix() {
        let blob = blob_of::<0, 4>(&[0xDE, 0xAD]);
        assert_eq!(format!("{blob:#x}"), "dead");
        assert_eq!(format!("{blob:#X}"), "DEAD");
    }

    #[test]
    fn vector_with_a_minimum_is_printed() {
        let blob = blob_of::<2, 4>(&[0xDE, 0xAD]);
        assert_eq!(format!("{blob:x}"), "dead");
    }
}

#[cfg(feature = "hex")]
mod hex_parse {
    use amplify_num::hex::{Error as HexError, FromHex};

    use super::*;

    #[test]
    fn digits_in_the_lower_case_are_decoded() {
        let blob = Blob::<0, 4>::from_hex("deadbeef").unwrap();
        assert_eq!(blob.as_slice(), [0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn digits_in_the_upper_case_are_decoded() {
        let blob = Blob::<0, 4>::from_hex("DEADBEEF").unwrap();
        assert_eq!(blob.as_slice(), [0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn digits_in_both_cases_are_decoded() {
        let blob = Blob::<0, 4>::from_hex("dEaDBeeF").unwrap();
        assert_eq!(blob.as_slice(), [0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn data_at_both_bounds_are_decoded() {
        assert_eq!(Blob::<2, 4>::from_hex("dead").unwrap().len(), 2);
        assert_eq!(Blob::<2, 4>::from_hex("deadbeef").unwrap().len(), 4);
    }

    #[test]
    fn data_below_the_minimum_report_the_minimum_and_their_length_in_bytes() {
        assert_eq!(
            Blob::<3, 4>::from_hex("dead"),
            Err(HexError::InvalidLength(3, 2))
        );
    }

    #[test]
    fn data_above_the_maximum_report_the_maximum_and_their_length_in_bytes() {
        assert_eq!(
            Blob::<0, 3>::from_hex("deadbeef"),
            Err(HexError::InvalidLength(3, 4))
        );
    }

    #[test]
    fn empty_string_is_decoded_without_a_minimum() {
        assert!(Blob::<0, 4>::from_hex("").unwrap().is_empty());
    }

    #[test]
    fn empty_string_is_rejected_by_a_minimum() {
        assert_eq!(
            Blob::<1, 4>::from_hex(""),
            Err(HexError::InvalidLength(1, 0))
        );
    }

    #[test]
    fn string_of_an_odd_length_is_rejected_with_its_length() {
        assert_eq!(
            Blob::<0, 4>::from_hex("dea"),
            Err(HexError::OddLengthString(3))
        );
    }

    #[test]
    fn odd_length_is_reported_before_the_bounds() {
        assert_eq!(
            Blob::<0, 1>::from_hex("deadbee"),
            Err(HexError::OddLengthString(7))
        );
    }

    #[test]
    fn other_characters_are_rejected_with_their_code() {
        assert_eq!(
            Blob::<0, 4>::from_hex("dexd"),
            Err(HexError::InvalidChar(b'x'))
        );
        assert_eq!(
            Blob::<0, 4>::from_hex("xead"),
            Err(HexError::InvalidChar(b'x'))
        );
        assert_eq!(
            Blob::<0, 4>::from_hex("de d"),
            Err(HexError::InvalidChar(b' '))
        );
    }

    #[test]
    fn bounds_are_reported_before_the_characters() {
        assert_eq!(
            Blob::<0, 1>::from_hex("zzzz"),
            Err(HexError::InvalidLength(1, 2))
        );
    }

    #[test]
    fn characters_outside_of_ascii_are_rejected_without_a_panic() {
        // The string takes two bytes, which are read as a pair of digits.
        assert_eq!(
            Blob::<0, 4>::from_hex("ñ"),
            Err(HexError::InvalidChar("ñ".as_bytes()[0]))
        );
    }

    #[test]
    fn printed_vector_is_decoded_back() {
        let blob = Blob::<0, 256>::try_from_iter(0..=u8::MAX).unwrap();
        assert_eq!(
            Blob::<0, 256>::from_hex(&format!("{blob:x}")),
            Ok(blob.clone())
        );
        assert_eq!(Blob::<0, 256>::from_hex(&format!("{blob:X}")), Ok(blob));
    }

    #[test]
    fn bytes_of_an_iterator_are_collected() {
        let bytes = vec![Ok(1), Ok(2), Ok(3)];
        let blob = Blob::<1, 4>::from_byte_iter(bytes.into_iter()).unwrap();
        assert_eq!(blob.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn too_few_bytes_of_an_iterator_report_the_minimum() {
        let bytes = vec![Ok(1)];
        assert_eq!(
            Blob::<2, 4>::from_byte_iter(bytes.into_iter()),
            Err(HexError::InvalidLength(2, 1))
        );
    }

    #[test]
    fn too_many_bytes_of_an_iterator_report_the_maximum_and_the_byte_above_it() {
        // The iterator is not read to its end, so its length is not known.
        let bytes = vec![Ok(1), Ok(2), Ok(3), Ok(4), Ok(5)];
        assert_eq!(
            Blob::<0, 2>::from_byte_iter(bytes.into_iter()),
            Err(HexError::InvalidLength(2, 3))
        );
    }

    #[test]
    fn error_of_an_iterator_is_passed_on() {
        let bytes = vec![Ok(1), Err(HexError::InvalidChar(b'x')), Ok(3)];
        assert_eq!(
            Blob::<0, 4>::from_byte_iter(bytes.into_iter()),
            Err(HexError::InvalidChar(b'x'))
        );
    }

    #[test]
    fn empty_iterator_is_collected_without_a_minimum() {
        let bytes: Vec<Result<u8, HexError>> = vec![];
        assert!(
            Blob::<0, 4>::from_byte_iter(bytes.into_iter())
                .unwrap()
                .is_empty()
        );
    }
}
