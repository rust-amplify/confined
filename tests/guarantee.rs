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

//! Regression tests for the guarantee of the crate: a confined collection
//! always holds between `MIN_LEN` and `MAX_LEN` items, whichever way it was
//! obtained or changed.

use std::cell::Cell;
#[cfg(feature = "std")]
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::panic::{self, AssertUnwindSafe};

#[cfg(feature = "indexmap")]
use confined::indexmap::IndexMap;
use confined::{Confined, Error, ItemLen};

#[test]
fn from_str_checks_limits() {
    type Short = Confined<String, 1, 3>;

    assert_eq!(
        "".parse::<Short>(),
        Err(Error::Undersize { len: 0, min_len: 1 })
    );
    assert_eq!(
        "way too long".parse::<Short>(),
        Err(Error::Oversize {
            len: 12,
            max_len: 3
        })
    );
    // The limits are in bytes: two characters, four bytes
    assert_eq!(
        "a€".parse::<Short>(),
        Err(Error::Oversize { len: 4, max_len: 3 })
    );
    assert_eq!("abc".parse::<Short>().unwrap().as_str(), "abc");
}

#[test]
#[cfg(feature = "ascii")]
fn ascii_from_str_checks_limits() {
    use confined::AsciiError;
    use confined::ascii::AsciiString;

    type Short = Confined<AsciiString, 1, 3>;

    assert_eq!(
        "abcd".parse::<Short>(),
        Err(AsciiError::Confinement(Error::Oversize {
            len: 4,
            max_len: 3
        }))
    );
    assert_eq!(
        "".parse::<Short>(),
        Err(AsciiError::Confinement(Error::Undersize {
            len: 0,
            min_len: 1
        }))
    );
    assert!(matches!("€".parse::<Short>(), Err(AsciiError::Ascii(_))));
    assert_eq!("abc".parse::<Short>().unwrap().as_str(), "abc");
}

#[test]
#[cfg(feature = "hex")]
fn from_hex_checks_limits() {
    use amplify_num::hex::{Error as HexError, FromHex};

    assert_eq!(
        Confined::<Vec<u8>, 0, 1>::from_hex("deadbeef"),
        Err(HexError::InvalidLength(1, 4))
    );
    assert_eq!(
        Confined::<Vec<u8>, 2, 4>::from_hex("de"),
        Err(HexError::InvalidLength(2, 1))
    );
    assert_eq!(
        Confined::<Vec<u8>, 1, 4>::from_hex(""),
        Err(HexError::InvalidLength(1, 0))
    );
    assert_eq!(
        Confined::<Vec<u8>, 0, 4>::from_hex("zz"),
        Err(HexError::InvalidChar(b'z'))
    );
    assert_eq!(
        Confined::<Vec<u8>, 0, 4>::from_hex("abc"),
        Err(HexError::OddLengthString(3))
    );

    let blob = Confined::<Vec<u8>, 0, 4>::from_hex("deadbeef").unwrap();
    assert_eq!(blob.as_slice(), &[0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn from_slice_checked_accepts_what_try_from_slice_accepts() {
    type UpToThree = Confined<Vec<u8>, 1, 3>;

    assert_eq!(UpToThree::from_slice_checked(&[7]).as_slice(), &[7]);
    assert_eq!(UpToThree::from_slice_checked(&[1, 2, 3]).as_slice(), &[
        1, 2, 3
    ]);
    assert!(Confined::<Vec<u8>, 0, 3>::from_slice_checked(&[]).is_empty());

    for slice in [&[][..], &[1, 2, 3, 4][..]] {
        assert!(UpToThree::try_from_slice(slice).is_err());
        assert!(panic::catch_unwind(|| UpToThree::from_slice_checked(slice)).is_err());
    }
}

#[test]
fn try_from_iter_stops_at_the_limit() {
    let read = Cell::new(0usize);
    let res = Confined::<Vec<u32>, 0, 2>::try_from_iter(
        (0..1_000_000u32).inspect(|_| read.set(read.get() + 1)),
    );
    assert_eq!(res, Err(Error::Oversize { len: 3, max_len: 2 }));
    assert!(read.get() <= 3, "{} items were read", read.get());

    // The same limits accept what fits
    assert_eq!(
        Confined::<Vec<u32>, 0, 2>::try_from_iter(0..2)
            .unwrap()
            .as_slice(),
        &[0, 1]
    );
    assert_eq!(
        Confined::<Vec<u32>, 2, 4>::try_from_iter(0..1),
        Err(Error::Undersize { len: 1, min_len: 2 })
    );
    // The length is the length of the collection, not the number of items read
    assert_eq!(
        Confined::<BTreeSet<u8>, 0, 2>::try_from_iter([1, 1, 1, 2])
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        Confined::<String, 0, 3>::try_from_iter("a€".chars()),
        Err(Error::Oversize { len: 4, max_len: 3 })
    );
}

#[test]
fn from_iter_checked_stops_at_the_limit() {
    let read = Cell::new(0usize);
    let res = panic::catch_unwind(AssertUnwindSafe(|| {
        Confined::<Vec<u32>, 0, 2>::from_iter_checked(
            (0..1_000_000u32).inspect(|_| read.set(read.get() + 1)),
        )
    }));
    assert!(res.is_err());
    assert!(read.get() <= 3, "{} items were read", read.get());

    assert_eq!(
        Confined::<Vec<u32>, 0, 2>::from_iter_checked(0..2).as_slice(),
        &[0, 1]
    );
}

#[test]
fn string_push_counts_bytes() {
    let mut s = Confined::<String, 0, 3>::try_from("ab".to_owned()).unwrap();
    assert_eq!(s.push('€'), Err(Error::Oversize { len: 2, max_len: 3 }));
    assert_eq!(s.as_str(), "ab");
    assert_eq!(s.push('c'), Ok(()));
    assert_eq!(s.as_str(), "abc");
    assert_eq!(s.push('d'), Err(Error::Oversize { len: 3, max_len: 3 }));

    // A character is accepted when all its bytes fit
    let mut s = Confined::<String, 0, 3>::new();
    assert_eq!(s.push('€'), Ok(()));
    assert_eq!(s.len(), 3);
}

#[test]
fn string_extend_counts_bytes() {
    let mut s = Confined::<String, 0, 3>::try_from("a".to_owned()).unwrap();
    assert_eq!(
        s.extend("b€c".chars()),
        Err(Error::Oversize { len: 2, max_len: 3 })
    );
    assert_eq!(s.as_str(), "ab");
}

#[test]
fn string_remove_counts_bytes() {
    let mut s = Confined::<String, 2, 10>::try_from("a€".to_owned()).unwrap();
    assert_eq!(s.remove(1), Err(Error::Undersize { len: 4, min_len: 2 }));
    assert_eq!(s.as_str(), "a€");
    // An index inside of a character and an index past the end
    assert_eq!(s.remove(2), Err(Error::OutOfBoundary { index: 2, len: 4 }));
    assert_eq!(s.remove(4), Err(Error::OutOfBoundary { index: 4, len: 4 }));

    assert_eq!(s.remove(0), Ok('a'));
    assert_eq!(s.as_str(), "€");
    assert_eq!(s.remove(0), Err(Error::Undersize { len: 3, min_len: 2 }));
    assert_eq!(s.as_str(), "€");
}

#[test]
fn with_holds_a_single_item() {
    assert_eq!(Confined::<String, 1, 4>::with('€').as_str(), "€");
    assert_eq!(Confined::<String, 1, 4>::with(char::MAX).len(), 4);
    assert_eq!(Confined::<Vec<u8>, 1, 1>::with(7).as_slice(), &[7]);
    assert_eq!(Confined::<BTreeSet<u8>, 1, 1>::with(7).len(), 1);
    assert_eq!(Confined::<BTreeMap<u8, u8>, 1, 1>::with((7, 8)).len(), 1);
    assert_eq!(
        Confined::<BTreeMap<u8, u8>, 1, 1>::with_key_value(7, 8).len(),
        1
    );
}

#[test]
fn deque_without_minimum_keeps_infallible_methods() {
    let mut deque = Confined::<VecDeque<u8>, 0, 5>::try_from_iter([1, 2, 3, 4, 5]).unwrap();
    assert_eq!(deque.pop_front(), Some(1));
    assert_eq!(deque.pop_back(), Some(5));
    deque.truncate(2);
    assert_eq!(deque.iter().copied().collect::<Vec<_>>(), [2, 3]);
    assert_eq!(deque.drain(..).collect::<Vec<_>>(), [2, 3]);
    assert!(deque.is_empty());
    assert_eq!(deque.pop_front(), None);
    assert_eq!(deque.pop_back(), None);
}

#[test]
fn deque_with_minimum_is_shortened_with_remove() {
    let mut deque = Confined::<VecDeque<u8>, 1, 3>::try_from_iter([1, 2]).unwrap();
    assert_eq!(deque.remove(0), Ok(1));
    assert_eq!(
        deque.remove(0),
        Err(Error::Undersize { len: 1, min_len: 1 })
    );
    assert_eq!(deque.len(), 1);
}

/// Generic code adds items with the `ItemLen` bound.
fn fill<C: ItemLen, const MAX: usize>(
    col: &mut Confined<C, 0, MAX>,
    items: impl IntoIterator<Item = C::Item>,
) -> Result<(), Error> {
    col.extend(items)
}

#[test]
fn generic_code_adds_items() {
    let mut vec = Confined::<Vec<u8>, 0, 2>::new();
    assert_eq!(fill(&mut vec, [1, 2]), Ok(()));
    assert_eq!(
        fill(&mut vec, [3]),
        Err(Error::Oversize { len: 2, max_len: 2 })
    );

    let mut s = Confined::<String, 0, 4>::new();
    assert_eq!(
        fill(&mut s, "€€".chars()),
        Err(Error::Oversize { len: 3, max_len: 4 })
    );
    assert_eq!(s.as_str(), "€");
}

/// The scenarios which are the same for each type of a map.
macro_rules! map_tests {
    ($name:ident, $map:ident) => {
        mod $name {
            use super::*;

            #[test]
            fn entry_inserts_within_limits() {
                let mut map = Confined::<$map<u8, u8>, 1, 2>::with_key_value(1, 10);

                // A present key: the old value is kept
                assert_eq!(*map.entry(1).unwrap().or_insert(0), 10);
                assert_eq!(map.entry(1).unwrap().key(), &1);
                // A new key with room: the value is inserted
                assert_eq!(*map.entry(2).unwrap().or_insert(20), 20);
                assert_eq!(map.len(), 2);
                // A new key on a full map
                assert_eq!(map.entry(3).unwrap_err(), Error::Oversize {
                    len: 2,
                    max_len: 2
                });
                assert_eq!(map.len(), 2);
                // A present key on a full map
                *map.entry(2).unwrap().or_default() += 1;
                assert_eq!(map[&2], 21);
            }

            #[test]
            fn entry_methods() {
                let mut map = Confined::<$map<u8, u8>, 0, 8>::new();
                assert_eq!(*map.entry(1).unwrap().or_insert_with(|| 10), 10);
                assert_eq!(
                    *map.entry(2).unwrap().or_insert_with_key(|key| key * 10),
                    20
                );
                assert_eq!(*map.entry(3).unwrap().or_default(), 0);

                let entry = map.entry(1).unwrap().and_modify(|value| *value += 1);
                assert_eq!(*entry.or_insert(0), 11);
                let entry = map.entry(4).unwrap().and_modify(|value| *value += 1);
                assert_eq!(*entry.or_insert(40), 40);
                assert_eq!(map.len(), 4);
            }

            #[test]
            fn try_retain_stops_at_the_minimum() {
                let mut map =
                    Confined::<$map<u8, u8>, 2, 5>::try_from_iter([(1, 1), (2, 2), (3, 3)])
                        .unwrap();
                let mut calls = 0;
                let res = map.try_retain(|_, _| {
                    calls += 1;
                    false
                });
                assert_eq!(res, Err(Error::Undersize { len: 0, min_len: 2 }));
                assert_eq!(map.len(), 2);
                assert_eq!(calls, 3);

                assert_eq!(map.try_retain(|_, _| true), Ok(()));
                assert_eq!(map.len(), 2);
            }

            #[test]
            fn try_retain_removes_down_to_the_minimum() {
                let mut map =
                    Confined::<$map<u8, u8>, 2, 5>::try_from_iter([(1, 1), (2, 2), (3, 3), (4, 4)])
                        .unwrap();
                assert_eq!(map.try_retain(|key, _| key % 2 == 0), Ok(()));
                assert_eq!(map.len(), 2);
                assert!(map.contains_key(&2) && map.contains_key(&4));

                // The predicate rejects one item, and the map can't lose any
                let res = map.try_retain(|key, _| *key == 2);
                assert_eq!(res, Err(Error::Undersize { len: 1, min_len: 2 }));
                assert!(map.contains_key(&2) && map.contains_key(&4));
            }
        }
    };
}

map_tests!(btree, BTreeMap);
#[cfg(feature = "std")]
map_tests!(hashed, HashMap);
#[cfg(feature = "indexmap")]
map_tests!(indexed, IndexMap);

#[test]
fn try_retain_removes_in_the_order_of_iteration() {
    let mut map =
        Confined::<BTreeMap<u8, u8>, 2, 5>::try_from_iter([(1, 1), (2, 2), (3, 3)]).unwrap();
    assert!(map.try_retain(|_, _| false).is_err());
    assert_eq!(map.keys().copied().collect::<Vec<_>>(), [2, 3]);
}

#[test]
fn append_counts_new_keys_only() {
    let mut map = Confined::<BTreeMap<u8, u8>, 0, 2>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    let mut other = Confined::<BTreeMap<u8, u8>, 0, 9>::try_from_iter([(2, 21)]).unwrap();
    assert_eq!(map.append(&mut other), Ok(()));
    assert_eq!(map.len(), 2);
    assert_eq!(map[&2], 21);
    assert!(other.is_empty());
}

#[test]
fn append_which_does_not_fit_changes_nothing() {
    let mut map = Confined::<BTreeMap<u8, u8>, 0, 2>::try_from_iter([(1, 10), (2, 20)]).unwrap();
    let mut other = Confined::<BTreeMap<u8, u8>, 0, 9>::try_from_iter([(2, 21), (3, 30)]).unwrap();
    assert_eq!(
        map.append(&mut other),
        Err(Error::Oversize { len: 3, max_len: 2 })
    );
    assert_eq!(map.len(), 2);
    assert_eq!(map[&2], 20);
    assert_eq!(other.len(), 2);
}

#[test]
#[cfg(feature = "std")]
fn hash_map_reserve_is_limited_by_the_confinement() {
    let mut map = Confined::<HashMap<u8, u8>, 0, 3>::try_from_iter([(1, 1)]).unwrap();
    map.reserve(usize::MAX);
    assert!(map.capacity() >= 3);

    let mut map =
        Confined::<HashMap<u16, u16>, 0, 1000>::try_from_iter((0..1000).map(|i| (i, i))).unwrap();
    let capacity = map.capacity();
    map.reserve(0);
    map.reserve(1000);
    assert_eq!(map.capacity(), capacity);
}

#[test]
fn first_mut_and_last_mut() {
    let mut vec = Confined::<Vec<u8>, 1, 3>::from_array([1, 2, 3]);
    *vec.first_mut() = 10;
    *vec.last_mut() = 30;
    assert_eq!(*vec.first(), 10);
    assert_eq!(*vec.last(), 30);
    assert_eq!(vec.as_slice(), &[10, 2, 30]);

    let mut deque = Confined::<VecDeque<u8>, 1, 3>::try_from_iter([1, 2, 3]).unwrap();
    *deque.first_mut() = 10;
    *deque.last_mut() = 30;
    assert_eq!(*deque.first(), 10);
    assert_eq!(*deque.last(), 30);
    assert_eq!(deque.iter().copied().collect::<Vec<_>>(), [10, 2, 30]);

    // A single item is both the first and the last one
    let mut single = Confined::<Vec<u8>, 1, 3>::with(1);
    *single.last_mut() += 1;
    assert_eq!(*single.first(), 2);
}

#[test]
#[cfg(feature = "ascii")]
fn ascii_first_and_last() {
    use confined::ascii::{AsciiChar, AsciiString};

    let mut s: Confined<AsciiString, 1, 8> = "abc".parse().unwrap();
    assert_eq!(*s.first(), AsciiChar::a);
    assert_eq!(*s.last(), AsciiChar::c);
    *s.first_mut() = AsciiChar::x;
    *s.last_mut() = AsciiChar::z;
    assert_eq!(s.as_str(), "xbz");
}

#[test]
#[cfg(feature = "indexmap")]
fn index_set_first_and_last() {
    use confined::indexmap::IndexSet;

    let set = Confined::<IndexSet<u8>, 1, 8>::try_from_iter([3, 1, 2]).unwrap();
    assert_eq!(*set.first(), 3);
    assert_eq!(*set.last(), 2);
}

#[test]
fn from_array_keeps_the_items() {
    assert_eq!(Confined::<Vec<u8>, 1, 3>::from_array([1]).as_slice(), &[1]);
    assert_eq!(
        Confined::<Vec<u8>, 1, 3>::from_array([1, 2, 3]).as_slice(),
        &[1, 2, 3]
    );
    assert!(Confined::<Vec<u8>, 0, 3>::from_array([]).is_empty());

    // The items do not have to be `Clone`, unlike in `try_from_slice`
    struct Unique(u8);
    let vec = Confined::<Vec<Unique>, 2, 2>::from_array([Unique(1), Unique(2)]);
    assert_eq!(vec[1].0, 2);
}

#[test]
fn widen_keeps_the_collection() {
    let narrow = Confined::<Vec<u8>, 1, 3>::from_array([1, 2]);
    let wide: Confined<Vec<u8>, 0, 255> = narrow.clone().widen();
    assert_eq!(wide.as_slice(), narrow.as_slice());
    assert_eq!(wide.len_u8(), 2);

    // The same bounds are accepted, as well as a change of a single bound
    assert_eq!(narrow.clone().widen::<1, 3>(), narrow);
    assert_eq!(narrow.clone().widen::<0, 3>().as_slice(), &[1, 2]);
    assert_eq!(narrow.clone().widen::<1, 4>().as_slice(), &[1, 2]);

    let map = Confined::<BTreeMap<u8, u8>, 1, 3>::with_key_value(1, 1);
    assert_eq!(map.widen::<0, { usize::MAX }>().len(), 1);
}

#[cfg(feature = "serde")]
mod serde_support {
    use std::fmt::{self, Display, Formatter};

    use confined::{Confined, Error};
    use serde::de::value::SeqDeserializer;
    use serde::de::{self, Deserialize, Deserializer, IntoDeserializer, Visitor};
    use serde::ser::{self, Impossible, Serialize, SerializeSeq, Serializer};

    /// The part of the serde data model which is used by a confined `Vec<u8>`.
    #[derive(Clone, PartialEq, Debug)]
    enum Token {
        U8(u8),
        Seq(Vec<Token>),
        Newtype(&'static str, Box<Token>),
    }

    #[derive(Debug)]
    struct TokenError(String);

    impl Display for TokenError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl ser::StdError for TokenError {}

    impl ser::Error for TokenError {
        fn custom<T: Display>(msg: T) -> Self {
            TokenError(msg.to_string())
        }
    }

    impl de::Error for TokenError {
        fn custom<T: Display>(msg: T) -> Self {
            TokenError(msg.to_string())
        }
    }

    struct TokenSerializer;

    struct SeqSerializer(Vec<Token>);

    impl SerializeSeq for SeqSerializer {
        type Ok = Token;
        type Error = TokenError;

        fn serialize_element<T: ?Sized + Serialize>(
            &mut self,
            value: &T,
        ) -> Result<(), TokenError> {
            self.0.push(value.serialize(TokenSerializer)?);
            Ok(())
        }

        fn end(self) -> Result<Token, TokenError> {
            Ok(Token::Seq(self.0))
        }
    }

    macro_rules! unsupported {
        ($($name:ident($($arg:ty),*) -> $ok:ty;)+) => {
            $(
                fn $name(self $(, _: $arg)*) -> Result<$ok, TokenError> {
                    Err(TokenError(format!("{} is not supported", stringify!($name))))
                }
            )+
        };
    }

    impl Serializer for TokenSerializer {
        type Ok = Token;
        type Error = TokenError;
        type SerializeSeq = SeqSerializer;
        type SerializeTuple = Impossible<Token, TokenError>;
        type SerializeTupleStruct = Impossible<Token, TokenError>;
        type SerializeTupleVariant = Impossible<Token, TokenError>;
        type SerializeMap = Impossible<Token, TokenError>;
        type SerializeStruct = Impossible<Token, TokenError>;
        type SerializeStructVariant = Impossible<Token, TokenError>;

        fn serialize_u8(self, v: u8) -> Result<Token, TokenError> {
            Ok(Token::U8(v))
        }

        fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer, TokenError> {
            Ok(SeqSerializer(Vec::with_capacity(len.unwrap_or_default())))
        }

        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            name: &'static str,
            value: &T,
        ) -> Result<Token, TokenError> {
            Ok(Token::Newtype(
                name,
                Box::new(value.serialize(TokenSerializer)?),
            ))
        }

        fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Token, TokenError> {
            Err(TokenError("serialize_some is not supported".to_owned()))
        }

        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            _: &'static str,
            _: u32,
            _: &'static str,
            _: &T,
        ) -> Result<Token, TokenError> {
            Err(TokenError(
                "serialize_newtype_variant is not supported".to_owned(),
            ))
        }

        unsupported! {
            serialize_bool(bool) -> Token;
            serialize_i8(i8) -> Token;
            serialize_i16(i16) -> Token;
            serialize_i32(i32) -> Token;
            serialize_i64(i64) -> Token;
            serialize_u16(u16) -> Token;
            serialize_u32(u32) -> Token;
            serialize_u64(u64) -> Token;
            serialize_f32(f32) -> Token;
            serialize_f64(f64) -> Token;
            serialize_char(char) -> Token;
            serialize_str(&str) -> Token;
            serialize_bytes(&[u8]) -> Token;
            serialize_none() -> Token;
            serialize_unit() -> Token;
            serialize_unit_struct(&'static str) -> Token;
            serialize_unit_variant(&'static str, u32, &'static str) -> Token;
            serialize_tuple(usize) -> Impossible<Token, TokenError>;
            serialize_tuple_struct(&'static str, usize) -> Impossible<Token, TokenError>;
            serialize_tuple_variant(&'static str, u32, &'static str, usize)
                -> Impossible<Token, TokenError>;
            serialize_map(Option<usize>) -> Impossible<Token, TokenError>;
            serialize_struct(&'static str, usize) -> Impossible<Token, TokenError>;
            serialize_struct_variant(&'static str, u32, &'static str, usize)
                -> Impossible<Token, TokenError>;
        }
    }

    impl<'de> IntoDeserializer<'de, TokenError> for Token {
        type Deserializer = Self;

        fn into_deserializer(self) -> Self {
            self
        }
    }

    impl<'de> Deserializer<'de> for Token {
        type Error = TokenError;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, TokenError> {
            match self {
                Token::U8(v) => visitor.visit_u8(v),
                Token::Seq(items) => visitor.visit_seq(SeqDeserializer::new(items.into_iter())),
                Token::Newtype(_, inner) => visitor.visit_newtype_struct(*inner),
            }
        }

        /// A newtype struct must be stored under the name with which it is
        /// read, as in the data formats keeping the names.
        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,
            name: &'static str,
            visitor: V,
        ) -> Result<V::Value, TokenError> {
            match self {
                Token::Newtype(stored, inner) if stored == name => {
                    visitor.visit_newtype_struct(*inner)
                }
                other => Err(TokenError(format!(
                    "expected newtype struct {name}, found {other:?}"
                ))),
            }
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct seq tuple tuple_struct map
            struct enum identifier ignored_any
        }
    }

    type UpToTwo = Confined<Vec<u8>, 0, 2>;

    fn serialized(bytes: &[u8]) -> Token {
        let items = bytes.iter().copied().map(Token::U8).collect();
        Token::Newtype("Confined", Box::new(Token::Seq(items)))
    }

    #[test]
    fn deserialize_checks_limits() {
        assert_eq!(
            UpToTwo::deserialize(serialized(&[1, 2]))
                .unwrap()
                .as_slice(),
            &[1, 2]
        );
        assert!(UpToTwo::deserialize(serialized(&[])).unwrap().is_empty());

        let err = UpToTwo::deserialize(serialized(&[1, 2, 3, 4])).unwrap_err();
        assert_eq!(err.0, Error::Oversize { len: 4, max_len: 2 }.to_string());

        let err = Confined::<Vec<u8>, 1, 2>::deserialize(serialized(&[])).unwrap_err();
        assert_eq!(err.0, Error::Undersize { len: 0, min_len: 1 }.to_string());
    }

    #[test]
    fn serialize_then_deserialize() {
        let value = UpToTwo::try_from(vec![1, 2]).unwrap();
        let tokens = value.serialize(TokenSerializer).unwrap();
        assert_eq!(tokens, serialized(&[1, 2]));
        assert_eq!(UpToTwo::deserialize(tokens).unwrap(), value);
    }
}
