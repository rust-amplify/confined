# Confined collections with compile-time size guarantees

![Build](https://github.com/rust-amplify/confined/actions/workflows/build.yml/badge.svg)
![Tests](https://github.com/rust-amplify/confined/actions/workflows/test.yml/badge.svg)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)
![MSRV](https://img.shields.io/badge/rustc-1.75%2B-orange)

`confined` is a small Rust library built around a single type:

```rust,ignore
pub struct Confined<C: Collection, const MIN_LEN: usize, const MAX_LEN: usize>(/* C */);
```

It wraps a collection `C` (a `Vec`, `String`, `VecDeque`, map or set) and keeps
the number of its elements within `MIN_LEN..=MAX_LEN`. The limits are part of
the type, so a function signature can say "a non-empty list", "at most 255
bytes" or "between one and four signers", and the caller cannot pass anything
else.

```rust
use confined::{Confined, Error};

/// One to three signers: never none, never four.
type Signers = Confined<Vec<&'static str>, 1, 3>;

fn main() -> Result<(), Error> {
    // The size is checked once, when the value is created...
    let mut signers = Signers::try_from(vec!["alice"])?;
    assert_eq!(Signers::try_from(vec![]), Err(Error::Undersize { len: 0, min_len: 1 }));

    // ...and again by every operation that changes it.
    signers.push("bob")?;
    signers.push("carol")?;
    assert_eq!(signers.push("dave"), Err(Error::Oversize { len: 3, max_len: 3 }));

    signers.remove(0)?;
    signers.remove(0)?;
    assert_eq!(signers.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));

    // Everything read-only comes from the inner `Vec` through `Deref`.
    assert_eq!(signers.len(), 1);
    assert!(signers.contains(&"carol"));

    // The list cannot be empty, so `first()` returns a value, not an `Option`.
    assert_eq!(*signers.first(), "carol");
    Ok(())
}
```

> **Project status.** The crate is at version 0.5.0 and is not published on
> crates.io yet. The code was split out of the `confinement` module of the
> [`amplify`](https://github.com/rust-amplify/rust-amplify) library. Version
> 0.5.0 changes parts of that API; [CHANGELOG.md](./CHANGELOG.md) lists every
> breaking change with its migration. What is still missing is listed under
> [Status and known limitations](#status-and-known-limitations).

## Contents

- [Why use it](#why-use-it)
- [What "compile-time" means here](#what-compile-time-means-here)
- [Installation](#installation)
- [Usage](#usage)
- [Supported collections](#supported-collections)
- [Behaviour worth knowing](#behaviour-worth-knowing)
- [Status and known limitations](#status-and-known-limitations)
- [Repository layout](#repository-layout)
- [Development](#development)
- [License](#license)

## Why use it

- **Size rules live in the type.** A field of type `Confined<Vec<T>, 1, 4>`
  documents and enforces its own limits; no comment or separate validation
  function can drift out of sync with it.
- **Validate once, at the boundary.** Code that receives a confined collection
  does not need to re-check its length.
- **Length prefixes cannot overflow.** If the maximum is `u16::MAX`, the length
  always fits a `u16`, and `len_u16()` returns it without a cast or an `unwrap`.
  This is handy for binary encodings and consensus-critical data.
- **Bounded input.** An upper limit caps how much untrusted data a structure
  can accumulate.
- **No memory overhead.** `Confined` is a newtype with the same size as the
  collection it wraps. The limits exist only in the type.

## What "compile-time" means here

The limits are const generic parameters, so they are fixed at compile time and
two different limits give two different types. The number of elements is only
known at run time, so the actual check happens at run time: on construction and
in every size-changing method. A violation is returned as an
[`Error`](#errors). The exceptions are the functions whose names end in
`_checked`, which panic instead.

The compiler contributes three things.

1. **The unchecked mutators are out of reach.** `Confined` implements `Deref`
   but not `DerefMut`, so all read-only methods of the inner collection are
   available, while its own `push`, `insert`, `remove` and similar methods are
   not. Only the checked replacements defined on `Confined` can change the
   size.
2. **Methods exist only for the limits that make them valid.** Calling one on
   a type with other limits is a compile error. The first table below lists
   them.
3. **Sizes known at build time are checked at build time.** Where every size
   involved is a constant, a wrong combination stops the build instead of
   returning an error at run time. The second table lists these checks.

| Available only when | Items                                                                                                                                                |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| `MIN_LEN` is `0`    | `new()`, `with_capacity()`, `Default`, `clear()`; `pop()` on `Vec`, `String` and `AsciiString`; `pop_front()`, `pop_back()`, `truncate()` and `drain()` on `VecDeque`; `retain()` on maps; `drain()` on `HashMap`; `io::Write` on `Vec<u8>` |
| `MIN_LEN` is `1`    | `with(item)`; `with_key_value(k, v)` on maps; `first()` and `last()` returning `&T` instead of `Option<&T>` on `Vec`, `VecDeque`, `AsciiString` and `IndexSet`; `first_mut()` and `last_mut()` returning `&mut T` on `Vec`, `VecDeque` and `AsciiString` |
| `MAX_LEN` is `U8`   | `len_u8() -> u8`                                                                                                                                     |
| `MAX_LEN` is `U16`  | `len_u16() -> u16`                                                                                                                                   |
| `MAX_LEN` is `U24`  | `len_u24() -> amplify_num::u24`                                                                                                                      |
| `MAX_LEN` is `U32`  | `len_u32() -> u32`                                                                                                                                   |

The match is exact: a type with `MIN_LEN = 2` is never empty either, but it
does not get `first()` and `last()`.

| Function                                    | The build fails unless                            |
|---------------------------------------------|---------------------------------------------------|
| `from_array([T; N])` on `Vec`               | `MIN_LEN <= N` and `N <= MAX_LEN`                 |
| `widen::<MIN2, MAX2>()`                     | `MIN2 <= MIN_LEN` and `MAX_LEN <= MAX2`           |
| `with(item)`                                | `MAX_LEN >= 1`, or `MAX_LEN >= 4` for a `String`  |
| `with_key_value(k, v)`                      | `MAX_LEN >= 1`                                    |
| `try_from` and the constructors built on it | `MIN_LEN <= MAX_LEN`                              |

For example, `Confined::<Vec<u8>, 1, 0>::with(7)` stops the build with the
message "the lower bound of the confinement exceeds its upper bound". These
checks run when machine code is generated: `cargo build` and `cargo test`
report them, but `cargo check` does not.

## Installation

Neither `confined` nor its `collections` dependency is on crates.io at the
moment, so both have to come from git. Cargo reads `[patch]` sections only from
the top-level manifest, which means the patch for `collections` must be
repeated in your own `Cargo.toml`:

```toml
[dependencies]
confined = { git = "https://github.com/rust-amplify/confined" }

[patch.crates-io]
collections = { git = "https://github.com/rust-amplify/collections" }
```

The minimum supported Rust version is 1.75.0.

### Cargo features

| Feature    | Default | What it adds                                                                                              |
|------------|:-------:|-----------------------------------------------------------------------------------------------------------|
| `std`      |   yes   | `HashMap` and `HashSet` support, `std::io::Write` for confined `Vec<u8>`, `std::error::Error` for `Error`  |
| `hex`      |   yes   | `{:x}` / `{:X}` formatting and `amplify_num::hex::FromHex` for confined `Vec<u8>`                          |
| `ascii`    |   no    | Support for `ascii::AsciiString` and the `AsciiError` type; re-exports the `ascii` crate as `confined::ascii` |
| `indexmap` |   no    | Support for `IndexMap` and `IndexSet`; re-exports the `indexmap` crate as `confined::indexmap`; enables `std` |
| `serde`    |   no    | `Serialize` and `Deserialize` for `Confined`; deserialization checks the limits                            |
| `all`      |   no    | Everything above                                                                                           |

The crate works in `no_std` + `alloc` environments when the default features
are disabled. Two combinations still need `std`; see
[limitations](#status-and-known-limitations).

## Usage

### Creating and releasing

```rust
use confined::Confined;

type UpToThree = Confined<Vec<u8>, 1, 3>;

let a = UpToThree::try_from(vec![1, 2])?; // from an owned collection
let b = UpToThree::try_from_iter([1, 2])?; // from an iterator
let c = UpToThree::try_from_slice(&[1, 2])?; // `Vec` only: clones a slice
let d = UpToThree::from_checked(vec![1, 2]); // panics instead of returning `Err`
let e = UpToThree::from_array([1, 2]); // `Vec` only: the length is checked by the build
assert!(a == b && b == c && c == d && d == e);

// A lower limit of 0 or 1 unlocks infallible constructors.
let empty = Confined::<Vec<u8>, 0, 3>::new();
let single = UpToThree::with(7);
assert!(empty.is_empty() && single.len() == 1);

// Moving to wider limits cannot fail, so it is not checked at run time.
let wide: Confined<Vec<u8>, 0, 255> = single.widen();
assert_eq!(wide.len_u8(), 1);

// Getting the plain collection back.
let borrowed: &Vec<u8> = a.as_unconfined();
assert_eq!(borrowed.len(), 2);
let cloned: Vec<u8> = a.to_unconfined();
let owned: Vec<u8> = a.release();
assert_eq!(cloned, owned);

Ok::<(), confined::Error>(())
```

`try_from_iter` stops reading the iterator as soon as the collection exceeds
`MAX_LEN`, so an oversized or endless iterator is rejected without being
collected. The `Oversize` error then carries the length at which it stopped.

`Confined::try_from` is an inherent function that takes the collection itself.
(It cannot be a `TryFrom` impl because of the blanket impl in `core`.) Strings
additionally implement `FromStr` and the real `TryFrom<&str>` trait. The
inherent function shadows the latter, so use `parse()` or `try_into()` for
string slices:

```rust
use confined::{Confined, Error};

type Name = Confined<String, 1, 32>;

let name: Name = "satoshi".parse()?;
assert_eq!(name.to_string(), "satoshi");
let same: Name = "satoshi".try_into()?;
assert_eq!(name, same);

assert_eq!("".parse::<Name>(), Err(Error::Undersize { len: 0, min_len: 1 }));

Ok::<(), Error>(())
```

### Modifying

```rust
use confined::{Confined, Error};

let mut v = Confined::<Vec<u8>, 0, 3>::new();
v.push(1)?;
v.extend([2, 3])?;
assert_eq!(v.push(4), Err(Error::Oversize { len: 3, max_len: 3 }));

// Changing items in place cannot break the limits, so it is unrestricted.
v[0] = 10;
for item in &mut v {
    *item += 1;
}
assert_eq!(v.as_slice(), &[11, 3, 4]);

assert_eq!(v.remove(7), Err(Error::OutOfBoundary { index: 7, len: 3 }));
assert_eq!(v.pop(), Some(4)); // exists only because `MIN_LEN` is 0
v.clear(); // same

// A list that is never empty gives direct access to both of its ends.
let mut v = Confined::<Vec<u8>, 1, 3>::from_array([1, 2]);
*v.first_mut() += 10;
*v.last_mut() += 20;
assert_eq!(v.as_slice(), &[11, 22]);

Ok::<(), Error>(())
```

### Maps and sets

```rust
use std::collections::BTreeMap;

use confined::{Confined, Error};

let mut votes = Confined::<BTreeMap<&str, u32>, 1, 2>::with_key_value("alice", 1);
votes.insert("bob", 1)?;

// Overwriting an existing key does not grow the map, so it works when full.
assert_eq!(votes.insert("alice", 5)?, Some(1));
assert_eq!(votes.insert("carol", 1), Err(Error::Oversize { len: 2, max_len: 2 }));

*votes.get_mut(&"bob").unwrap() += 1;
assert_eq!(votes[&"bob"], 2);

// `entry()` checks the upper limit itself, so `or_insert` cannot fail.
*votes.entry("bob")?.or_insert(0) += 1;
assert_eq!(votes[&"bob"], 3);
assert!(votes.entry("carol").is_err()); // a new key, and the map is full

// Removing a missing key is not an error. Removing the last allowed entry is.
assert_eq!(votes.remove(&"dave")?, None);
assert_eq!(votes.remove(&"bob")?, Some(3));
assert_eq!(votes.remove(&"alice"), Err(Error::Undersize { len: 1, min_len: 1 }));

Ok::<(), Error>(())
```

`entry()` returns `confined::Entry`, not the entry type of the wrapped map. It
has `key`, `or_insert`, `or_insert_with`, `or_insert_with_key`, `or_default`
and `and_modify`, and nothing that removes the entry: removal goes through
`remove` on the map, which checks the lower limit.

Sets use the generic `push` to add an item and `remove` / `take` to delete one.

### Size constants, aliases and macros

The crate exports constants for the common limits:

| Constant | Value                                                       |
|----------|-------------------------------------------------------------|
| `ZERO`   | `0`                                                         |
| `ONE`    | `1`                                                         |
| `U8`     | `u8::MAX` (255)                                             |
| `U16`    | `u16::MAX` (65 535)                                         |
| `U24`    | 2²⁴ − 1 (16 777 215)                                        |
| `U32`    | `u32::MAX`                                                  |
| `U64`    | `u64::MAX as usize`, which is `usize::MAX` on 64-bit targets |

They are what the `len_u8()` family of methods is keyed on, and what the
ready-made aliases are built from. The name of an alias is a size prefix
followed by a collection suffix, for example `TinyVec<T>` or
`NonEmptyOrdMap<K, V>`:

| Prefix     | Limits                                                        |
|------------|---------------------------------------------------------------|
| `Tiny`     | `0..=U8`                                                      |
| `Small`    | `0..=U16`                                                     |
| `Medium`   | `0..=U24`                                                     |
| `Large`    | `0..=U32`                                                     |
| `NonEmpty` | `1..=MAX`, where `MAX` is a parameter with the default `U64`  |
| `Confined` | `MIN..=MAX`, both parameters, with the defaults `0` and `U64` |

| Suffix                 | Collection                      | Feature    |
|------------------------|---------------------------------|------------|
| `String`               | `String`                        |            |
| `Ascii`                | `AsciiString`                   | `ascii`    |
| `Blob`                 | `Vec<u8>`                       |            |
| `Vec`                  | `Vec<T>`                        |            |
| `Deque`                | `VecDeque<T>`                   |            |
| `OrdSet`, `OrdMap`     | `BTreeSet<T>`, `BTreeMap<K, V>` |            |
| `HashSet`, `HashMap`   | `HashSet<T>`, `HashMap<K, V>`   | `std`      |
| `IndexSet`, `IndexMap` | `IndexSet<T>`, `IndexMap<K, V>` | `indexmap` |

`confined::SmallVec<T>` is therefore a `Vec` of up to 65 535 items. It is not
related to the `smallvec` crate.

```rust
use confined::{NonEmptyVec, SmallString, TinyBlob};

/// Writes a blob prefixed with its length as a single byte.
fn encode(blob: &TinyBlob, out: &mut Vec<u8>) {
    out.push(blob.len_u8()); // the type guarantees that the length fits
    out.extend_from_slice(blob.as_slice());
}

let blob = TinyBlob::try_from(vec![0xCA, 0xFE])?;
let mut out = vec![];
encode(&blob, &mut out);
assert_eq!(out, [2, 0xCA, 0xFE]);

let text = SmallString::try_from("hello".to_owned())?;
assert_eq!(text.len_u16(), 5);

let items = NonEmptyVec::<u8, 3>::try_from_iter([1, 2, 3])?;
assert_eq!(*items.last(), 3);

Ok::<(), confined::Error>(())
```

An alias of your own takes one line, for example
`type Signers = Confined<Vec<PublicKey>, 1, 15>;`.

The `Tiny`, `Small` and `Medium` aliases have constructor macros. Their names
start with `tiny_`, `small_` or `medium_`:

| Macro suffix     | Builds                            | Feature    |
|------------------|-----------------------------------|------------|
| `s!`             | a `String` from a string literal  |            |
| `blob!`          | a `Vec<u8>`                       |            |
| `vec!`           | a `Vec<T>`                        |            |
| `bset!`, `bmap!` | a `BTreeSet<T>`, `BTreeMap<K, V>` |            |
| `set!`, `map!`   | a `HashSet<T>`, `HashMap<K, V>`   | `std`      |
| `iset!`, `imap!` | an `IndexSet<T>`, `IndexMap<K, V>` | `indexmap` |

```rust
use confined::{small_s, tiny_blob, tiny_bmap, tiny_vec, TinyVec};

let names = tiny_vec!["alice", "bob"];
assert_eq!(names.len_u8(), 2);

let zeroes = tiny_blob![0u8; 32];
assert_eq!(zeroes.len_u8(), 32);

let greeting = small_s!("hello");
assert_eq!(greeting.len_u16(), 5);

let ages = tiny_bmap! { "alice" => 30u8, "bob" => 25 };
assert_eq!(ages[&"bob"], 25);

let empty: TinyVec<u8> = tiny_vec![];
assert!(empty.is_empty());
```

A macro panics if its content does not fit the limits of the type it builds.

### Byte strings

With the default `std` and `hex` features a confined `Vec<u8>` can be used as a
bounded write buffer, printed as hex and parsed from hex:

```rust
use std::io::Write;

use amplify_num::hex::{self, FromHex};
use confined::Confined;

let mut blob = Confined::<Vec<u8>, 0, 4>::new();
blob.write_all(&[0xDE, 0xAD, 0xBE, 0xEF])?;
assert!(blob.write_all(&[0x00]).is_err()); // does not fit, nothing is written
assert_eq!(format!("{blob:x}"), "deadbeef");
assert_eq!(format!("{blob:X}"), "DEADBEEF");

assert_eq!(Confined::<Vec<u8>, 0, 4>::from_hex("deadbeef"), Ok(blob));
assert_eq!(
    Confined::<Vec<u8>, 0, 3>::from_hex("deadbeef"),
    Err(hex::Error::InvalidLength(3, 4)) // the limit and the actual length
);

Ok::<(), std::io::Error>(())
```

A `write` call either stores the whole buffer or fails with
`io::ErrorKind::OutOfMemory`. The `FromHex` trait and its error type come from
the [`amplify_num`](https://crates.io/crates/amplify_num) crate.

### ASCII strings

With the `ascii` feature:

```rust
use confined::ascii::AsciiString;
use confined::{AsciiError, Confined, Error};

type Ticker = Confined<AsciiString, 1, 8>;

let ticker: Ticker = "BTC".parse()?;
assert_eq!(ticker.len(), 3);
assert!(Ticker::try_from(AsciiString::new()).is_err()); // too short

// A string slice can fail in two ways.
assert!(matches!("₿".parse::<Ticker>(), Err(AsciiError::Ascii(_))));
assert_eq!(
    "BITCOIN-USD".parse::<Ticker>(),
    Err(AsciiError::Confinement(Error::Oversize { len: 11, max_len: 8 }))
);

Ok::<(), AsciiError>(())
```

### Errors

```rust,ignore
pub enum Error {
    /// The operation would leave fewer than `MIN_LEN` items.
    Undersize { len: usize, min_len: usize },
    /// The operation would leave more than `MAX_LEN` items.
    Oversize { len: usize, max_len: usize },
    /// Index-based removal with an index past the end.
    OutOfBoundary { index: usize, len: usize },
}
```

`Error` is `Copy`, implements `Display`, and implements `std::error::Error`
when the `std` feature is on.

### Serde

With the `serde` feature `Confined` implements `Serialize` and `Deserialize`
as a newtype, so in formats like JSON it looks exactly like the inner
collection. Deserialization checks the limits: data with too few or too many
items are rejected with a custom error of the deserializer, which carries the
message of the matching [`Error`](#errors).

## Supported collections

`Confined` can wrap any type implementing the `Collection` trait of the
[`collections`](https://github.com/rust-amplify/collections) crate. That crate
implements it for the types below. Map types also implement `KeyedCollection`.

| Collection       | Feature          | Checked methods specific to it                                                                                                |
|------------------|------------------|-------------------------------------------------------------------------------------------------------------------------------|
| `Vec<T>`         |                  | `from_array`, `try_from_slice`, `from_slice_checked`, `as_slice`, `into_vec`, `iter`, `remove`, `pop`¹                          |
| `Vec<u8>`        | `std`, `hex`     | `std::io::Write`¹ (`std`); `LowerHex`, `UpperHex`, `FromHex` (`hex`)                                                            |
| `String`         |                  | `TryFrom<&str>`, `FromStr`, `remove`, `pop`¹                                                                                  |
| `AsciiString`    | `ascii`          | `TryFrom<&str>`, `FromStr`, `remove`, `pop`¹                                                                                  |
| `VecDeque<T>`    |                  | `push_front`, `push_back`, `remove`, `pop_front`¹, `pop_back`¹, `drain`¹, `truncate`¹                                          |
| `BTreeSet<T>`    |                  | `remove`, `take`                                                                                                              |
| `HashSet<T>`     | `std`            | `remove`, `take`                                                                                                              |
| `IndexSet<T>`    | `indexmap`       | `remove`, `take`                                                                                                              |
| `BTreeMap<K, V>` |                  | `remove`, `remove_entry`, `pop_first`, `pop_last`, `append`, `split_off`, `range_mut`, `into_keys`, `into_values`              |
| `HashMap<K, V>`  | `std`            | `remove`, `remove_entry`, `reserve`, `shrink_to_fit`, `shrink_to`, `into_keys`, `into_values`, `drain`¹                        |
| `IndexMap<K, V>` | `indexmap`       | `remove`, `into_keys`, `into_values`                                                                                          |

¹ Only when `MIN_LEN` is `0`.

`BTreeMap` keys must implement `Hash` in addition to `Ord`, because
`KeyedCollection` requires it.

`push`, `extend` and `with` also need the `ItemLen` trait of this crate, which
tells how much one item adds to the length. It is implemented for all the
collections above. A collection of your own needs `impl ItemLen for MyCollection {}`
if every item counts as one.

On top of that, every confined collection gets the generic API:

| Purpose           | Items                                                                                                                              |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------|
| Create            | `try_from`, `try_from_iter`, `from_checked`, `from_iter_checked`                                                                    |
| Change limits     | `widen` to limits that include the current ones                                                                                    |
| Read              | `Deref<Target = C>`, `as_unconfined`, `to_unconfined`, `AsRef<[T]>`, `Borrow<[T]>`, `Index`, `Display`, iteration over `&Confined`     |
| Change items      | `IndexMut`, `AsMut<[T]>`, `BorrowMut<[T]>`, `iter_mut`, iteration over `&mut Confined`                                               |
| Change size       | `push`, `extend`                                                                                                                   |
| Unwrap            | `release`, by-value `IntoIterator`                                                                                                 |
| Maps              | `insert`, `entry`, `get_mut`, `values_mut`, `keyed_values_mut`, `try_retain`, `retain`¹                                             |
| Re-validate       | `with_mut(f)` returns `Err` and `as_mut_checked(f)` panics if the limits do not hold after the closure has run                      |
| Derived traits    | `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, whenever the inner collection implements them                     |

## Behaviour worth knowing

- **Length is whatever the collection calls its length.** For `String` that is
  bytes, not characters: `"añb"` has three characters but needs `MAX_LEN >= 4`.
  `push` and `remove` count the bytes of the character they add or take away,
  so `push('€')` needs three free bytes. `with()` needs `MAX_LEN >= 4` for a
  `String`, whatever the character is, because the check is made at build time
  and a character may take four bytes.
- **`extend` is not atomic.** It pushes items one by one and stops at the first
  one that does not fit. The items before it stay in the collection.
- **Sets are checked before insertion.** `push` on a full set returns
  `Oversize` even if the item is already present. Maps are more precise:
  `insert` and `entry` for an existing key always succeed.
- **Removing something absent is not an error.** `remove` and `take` on maps and
  sets return `Ok(None)` or `Ok(false)`, even at the minimum size.
- **`String::remove` needs a character boundary.** An index inside a multi-byte
  character returns `OutOfBoundary` instead of panicking.
- **`IndexMap` and `IndexSet` removal preserves order.** It uses the `shift_*`
  operations.
- **`try_retain` stops at the minimum.** It removes rejected entries in
  iteration order while the map stays above `MIN_LEN`. If the predicate rejects
  more than that, the remaining ones are kept and the call returns `Undersize`
  with the length the map would have had. The predicate is still called once
  for every entry.
- **`BTreeMap::append` needs a source without a minimum.** It empties the other
  map, so that map must have `MIN_LEN = 0`. Keys present in both maps do not
  count towards `MAX_LEN`. If the result does not fit, neither map is changed.
- **`HashMap::reserve` is capped.** It reserves room for at most
  `MAX_LEN - len` more entries, whatever amount is asked for.

## Status and known limitations

- **Not on crates.io.** The crate and its `collections` dependency have to be
  taken from git; see [Installation](#installation).
- **`indexmap` needs `std`.** The feature enables `std` itself, because the
  `collections` crate supports `IndexMap` and `IndexSet` only with the default
  hasher.
- **`ascii` with `serde` needs `std`.** Without it the build fails inside the
  `ascii` crate (version 1.1.0).
- **Rust 1.75.0 with the `indexmap` feature needs an older `indexmap`.**
  Version 2.11.4 builds with that toolchain and 2.14.2 does not. The
  `Cargo.lock` of this repository pins 2.11.4; a project that depends on
  `confined` has to pin it itself, for example with
  `cargo update -p indexmap --precise 2.11.4`.
- **`Entry` has methods only for the built-in maps.** A map type of your own
  gets `entry()`, but the returned `Entry` has no methods for it.
- **`smallvec` is not supported.** The `amplify` version of this code had
  aliases and macros for `SmallVec`; the `collections` crate has no
  implementation for it.

## Repository layout

```text
src/
├── lib.rs           crate root: public re-exports and the size constants
├── base.rs          the `Confined` type and everything generic over the collection
├── specifics.rs     methods that exist only for particular limits (MIN 0, MIN 1, MAX U8...)
├── error.rs         the `Error` enum
├── aliases.rs       type aliases: TinyVec, SmallString, NonEmptyOrdMap...
├── macros.rs        constructor macros: tiny_vec!, small_s!, tiny_bmap!...
└── imp/             methods for one concrete collection type each
    ├── vec.rs       Vec<T>
    ├── bytes.rs     Vec<u8>: io::Write and hex
    ├── string.rs    String
    ├── ascii.rs     AsciiString            (feature `ascii`)
    ├── deque.rs     VecDeque<T>
    ├── btree.rs     BTreeSet, BTreeMap
    ├── hashed.rs    HashSet, HashMap       (feature `std`)
    └── indexed.rs   IndexSet, IndexMap     (feature `indexmap`)
tests/
├── confinement.rs   tests of the public API
└── guarantee.rs     regression tests: no value can get outside its limits
```

The examples in this file are compiled and run by `cargo test --all-features`.

## Development

```console
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo +nightly fmt --all
cargo doc --all-features --open
```

Formatting needs a nightly toolchain because `.rustfmt.toml` uses unstable
options. CI (`.github/workflows`) checks every feature on its own, Linux, macOS
and Windows on x86-64 and ARM, the stable, beta, nightly and 1.75.0 toolchains,
and runs the tests on `wasm32` in headless Chrome.

Contributions are welcome. Please read [CONTRIBUTING.md](./CONTRIBUTING.md)
first; commits must be signed off (`git commit -s`) under the [DCO](./DCO).
Security issues should be reported privately as described in
[SECURITY.md](./SECURITY.md). Participation is governed by the
[Code of Conduct](./CODE_OF_CONDUCT.md).

## License

Licensed under the [Apache License, Version 2.0](./LICENSE).

Written by Dr Maxim Orlovsky and maintained by the Laboratories for Ubiquitous
Deterministic Computing (UBIDECO), Institute for Distributed and Cognitive
Systems (InDCS), Switzerland. See [MAINTAINERS.md](./MAINTAINERS.md).
