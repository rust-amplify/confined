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

> **Project status.** The crate is at version 0.4.10 and is not published on
> crates.io yet. The code was recently split out of the `confinement` module of
> the [`amplify`](https://github.com/rust-amplify/rust-amplify) library and the
> migration is unfinished. Read [Status and known limitations](#status-and-known-limitations)
> before relying on it.

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

The compiler contributes two things.

1. **The unchecked mutators are out of reach.** `Confined` implements `Deref`
   but not `DerefMut`, so all read-only methods of the inner collection are
   available, while its own `push`, `insert`, `remove` and similar methods are
   not. Only the checked replacements defined on `Confined` can change the
   size.
2. **Methods exist only for the limits that make them valid.** Calling one on
   a type with other limits is a compile error.

| Available only when | Items                                                                                                                                                |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| `MIN_LEN` is `0`    | `new()`, `with_capacity()`, `Default`, `clear()`; `pop()` on `Vec`, `String` and `AsciiString`; `retain()` on maps; `drain()` on `HashMap`; `io::Write` on `Vec<u8>` |
| `MIN_LEN` is `1`    | `with(item)`; `with_key_value(k, v)` on maps; `first()` and `last()` returning `&T` instead of `Option<&T>` on `Vec`, `VecDeque` and other collections indexable by position |
| `MAX_LEN` is `U8`   | `len_u8() -> u8`                                                                                                                                     |
| `MAX_LEN` is `U16`  | `len_u16() -> u16`                                                                                                                                   |
| `MAX_LEN` is `U24`  | `len_u24() -> amplify_num::u24`                                                                                                                      |
| `MAX_LEN` is `U32`  | `len_u32() -> u32`                                                                                                                                   |

The match is exact: a type with `MIN_LEN = 2` is never empty either, but it
does not get `first()` and `last()`.

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
| `ascii`    |   no    | Support for `ascii::AsciiString`; re-exports the `ascii` crate as `confined::ascii`                        |
| `indexmap` |   no    | Support for `IndexMap` and `IndexSet`; re-exports the `indexmap` crate as `confined::indexmap`             |
| `serde`    |   no    | `Serialize` and `Deserialize` for `Confined`                                                               |
| `all`      |   no    | Everything above                                                                                           |

The crate is written for `no_std` + `alloc`, but a build without the `std`
feature does not compile at the moment (see
[limitations](#status-and-known-limitations)).

## Usage

### Creating and releasing

```rust
use confined::Confined;

type UpToThree = Confined<Vec<u8>, 1, 3>;

let a = UpToThree::try_from(vec![1, 2])?; // from an owned collection
let b = UpToThree::try_from_iter([1, 2])?; // from an iterator
let c = UpToThree::try_from_slice(&[1, 2])?; // `Vec` only: clones a slice
let d = UpToThree::from_checked(vec![1, 2]); // panics instead of returning `Err`
assert!(a == b && b == c && c == d);

// A lower limit of 0 or 1 unlocks infallible constructors.
let empty = Confined::<Vec<u8>, 0, 3>::new();
let single = UpToThree::with(7);
assert!(empty.is_empty() && single.len() == 1);

// Getting the plain collection back.
let borrowed: &Vec<u8> = a.as_unconfined();
assert_eq!(borrowed.len(), 2);
let cloned: Vec<u8> = a.to_unconfined();
let owned: Vec<u8> = a.release();
assert_eq!(cloned, owned);

Ok::<(), confined::Error>(())
```

`Confined::try_from` is an inherent function that takes the collection itself.
(It cannot be a `TryFrom` impl because of the blanket impl in `core`.) Strings
additionally implement the real `TryFrom<&str>` trait, which the inherent
function shadows, so use `try_into()` for string slices:

```rust
use confined::Confined;

type Name = Confined<String, 1, 32>;

let name: Name = "satoshi".try_into()?;
assert_eq!(name.to_string(), "satoshi");

Ok::<(), confined::Error>(())
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

// Removing a missing key is not an error. Removing the last allowed entry is.
assert_eq!(votes.remove(&"dave")?, None);
assert_eq!(votes.remove(&"bob")?, Some(2));
assert_eq!(votes.remove(&"alice"), Err(Error::Undersize { len: 1, min_len: 1 }));

Ok::<(), Error>(())
```

Sets use the generic `push` to add an item and `remove` / `take` to delete one.

### Size constants and your own aliases

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

They make short, readable aliases, and they are what the `len_u8()` family of
methods is keyed on:

```rust
use confined::{Confined, ONE, U16, U64, U8, ZERO};

type TinyBlob = Confined<Vec<u8>, ZERO, U8>; // up to 255 bytes
type SmallString = Confined<String, ZERO, U16>; // up to 65 535 bytes
type NonEmptyVec<T, const MAX: usize = U64> = Confined<Vec<T>, ONE, MAX>;

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

The crate itself does not export such aliases yet; see
[limitations](#status-and-known-limitations).

### Byte strings

With the default `std` and `hex` features a confined `Vec<u8>` can be used as a
bounded write buffer and printed as hex:

```rust
use std::io::Write;

use confined::Confined;

let mut blob = Confined::<Vec<u8>, 0, 4>::new();
blob.write_all(&[0xDE, 0xAD, 0xBE, 0xEF])?;
assert!(blob.write_all(&[0x00]).is_err()); // does not fit, nothing is written
assert_eq!(format!("{blob:x}"), "deadbeef");
assert_eq!(format!("{blob:X}"), "DEADBEEF");

Ok::<(), std::io::Error>(())
```

A `write` call either stores the whole buffer or fails with
`io::ErrorKind::OutOfMemory`.

### ASCII strings

With the `ascii` feature:

```rust
use confined::ascii::AsciiString;
use confined::Confined;

type Ticker = Confined<AsciiString, 1, 8>;

let ticker: Ticker = "BTC".try_into().unwrap();
assert_eq!(ticker.len(), 3);
assert!(Ticker::try_from(AsciiString::new()).is_err()); // too short
assert!(<Ticker as TryFrom<&str>>::try_from("₿").is_err()); // not ASCII
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

With the `serde` feature `Confined` derives `Serialize` and `Deserialize` as a
newtype, so in formats like JSON it looks exactly like the inner collection.
Deserialization does not validate the limits yet; see
[limitations](#status-and-known-limitations).

## Supported collections

`Confined` can wrap any type implementing the `Collection` trait of the
[`collections`](https://github.com/rust-amplify/collections) crate. That crate
implements it for the types below. Map types also implement `KeyedCollection`.

| Collection       | Feature          | Checked methods specific to it                                                                                                |
|------------------|------------------|-------------------------------------------------------------------------------------------------------------------------------|
| `Vec<T>`         |                  | `try_from_slice`, `from_slice_checked`, `as_slice`, `into_vec`, `iter`, `remove`, `pop`¹                                        |
| `Vec<u8>`        | `std`, `hex`     | `std::io::Write`¹ (`std`); `LowerHex`, `UpperHex`, `FromHex` (`hex`)                                                            |
| `String`         |                  | `TryFrom<&str>`, `remove`, `pop`¹                                                                                             |
| `AsciiString`    | `ascii`          | `TryFrom<&str>`, `remove`, `pop`¹                                                                                             |
| `VecDeque<T>`    |                  | `push_front`, `push_back`, `pop_front`, `pop_back`, `remove`, `drain`, `truncate`                                              |
| `BTreeSet<T>`    |                  | `remove`, `take`                                                                                                              |
| `HashSet<T>`     | `std`            | `remove`, `take`                                                                                                              |
| `IndexSet<T>`    | `indexmap`       | `remove`, `take`                                                                                                              |
| `BTreeMap<K, V>` |                  | `remove`, `remove_entry`, `pop_first`, `pop_last`, `append`, `split_off`, `range_mut`, `into_keys`, `into_values`              |
| `HashMap<K, V>`  | `std`            | `remove`, `remove_entry`, `reserve`, `shrink_to_fit`, `shrink_to`, `into_keys`, `into_values`, `drain`¹                        |
| `IndexMap<K, V>` | `indexmap`       | `remove`, `into_keys`, `into_values`                                                                                          |

¹ Only when `MIN_LEN` is `0`.

`BTreeMap` keys must implement `Hash` in addition to `Ord`, because
`KeyedCollection` requires it.

On top of that, every confined collection gets the generic API:

| Purpose           | Items                                                                                                                              |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------|
| Create            | `try_from`, `try_from_iter`, `from_checked`, `from_iter_checked`                                                                    |
| Read              | `Deref<Target = C>`, `as_unconfined`, `to_unconfined`, `AsRef<[T]>`, `Borrow<[T]>`, `Index`, `Display`, iteration over `&Confined`     |
| Change items      | `IndexMut`, `AsMut<[T]>`, `BorrowMut<[T]>`, `iter_mut`, iteration over `&mut Confined`                                               |
| Change size       | `push`, `extend`                                                                                                                   |
| Unwrap            | `release`, by-value `IntoIterator`                                                                                                 |
| Maps              | `insert`, `entry`, `get_mut`, `values_mut`, `keyed_values_mut`, `try_retain`                                                        |
| Re-validate       | `with_mut(f)` returns `Err` and `as_mut_checked(f)` panics if the limits do not hold after the closure has run                      |
| Derived traits    | `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, whenever the inner collection implements them                     |

A few older names are kept as deprecated aliases: `from_collection_unsafe`,
`from_iter_unsafe`, `from_slice_unsafe`, `as_inner`, `to_inner`, `into_inner`,
`unbox` and `push_from`.

## Behaviour worth knowing

- **Length is whatever the collection calls its length.** For `String` that is
  bytes, not characters: `"añb"` has three characters but needs `MAX_LEN >= 4`.
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
- **`BTreeMap::append` checks the sum of both lengths.** It can report
  `Oversize` when the maps share keys and the merged result would have fit.

## Status and known limitations

The points below were checked against the current `master`. They are gaps in
the present state of the code, not design decisions, and this section should
shrink as they are fixed.

**Paths that skip the size check.** A value produced this way can be outside
its limits, so validate it yourself, for example by passing
`value.release()` through `try_from` again.

- `FromStr` (`"...".parse()`), `FromHex` and serde's `Deserialize` construct
  the wrapper without checking the limits.
- On `VecDeque`, `pop_front`, `pop_back`, `truncate` and `drain` ignore
  `MIN_LEN`.
- `BTreeMap::append` empties the other map whatever its `MIN_LEN` is.
- `try_retain` returns `Undersize` after the items have already been removed,
  so the map stays below its minimum.

**Other rough edges.**

- Building without the `std` feature fails (for one, `src/imp/string.rs` lacks
  a `ToOwned` import), so `no_std` does not work yet.
- `from_slice_checked` panics when the slice length equals `MIN_LEN`, including
  an empty slice for `MIN_LEN = 0`. `try_from_slice` is not affected.
- `AsciiError`, the error type of `TryFrom<&str>` for confined `AsciiString`,
  is not re-exported and cannot be named outside the crate.
- `src/confinement.rs` is the original single-file module from `amplify`. It
  is not part of the module tree and is not compiled. It still holds what has
  not been ported: the type aliases (`TinyVec`, `SmallString`, `NonEmptyVec`
  and so on), the constructor macros (`tiny_vec!`, `small_bmap!` and so on) and
  the unit tests. As a result `cargo test` currently runs no tests.
- Deprecation notes say "since 4.7.0". That is a version of `amplify`, not of
  this crate.

## Repository layout

```text
src/
├── lib.rs           crate root: public re-exports and the size constants
├── base.rs          the `Confined` type and everything generic over the collection
├── specifics.rs     methods that exist only for particular limits (MIN 0, MIN 1, MAX U8...)
├── error.rs         the `Error` enum
├── imp/             methods for one concrete collection type each
│   ├── vec.rs       Vec<T>
│   ├── bytes.rs     Vec<u8>: io::Write and hex
│   ├── string.rs    String
│   ├── ascii.rs     AsciiString            (feature `ascii`)
│   ├── deque.rs     VecDeque<T>
│   ├── btree.rs     BTreeSet, BTreeMap
│   ├── hashed.rs    HashSet, HashMap       (feature `std`)
│   └── indexed.rs   IndexSet, IndexMap     (feature `indexmap`)
└── confinement.rs   legacy single-file version, not compiled
```

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
