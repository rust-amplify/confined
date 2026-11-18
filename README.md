# Confined collections with compile-time size guarantees

![Build](https://github.com/rust-amplify/confined/actions/workflows/build.yml/badge.svg)
![Tests](https://github.com/rust-amplify/confined/actions/workflows/test.yml/badge.svg)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](./LICENSE)
![MSRV](https://img.shields.io/badge/rustc-1.100.0%2B-orange)

`confined` is a small Rust library built around a single type:

```rust,ignore
pub struct Confined<C, const MIN_LEN: usize, const MAX_LEN: usize>(/* C */);
```

It wraps a collection `C` (a `Vec`, `String`, `VecDeque`, `LinkedList`,
`BinaryHeap`, map or set) and keeps the number of its elements within
`MIN_LEN..=MAX_LEN`. The limits are part of the type, so a function signature
can say "a non-empty list", "at most 255 bytes" or "between one and four
signers", and the caller cannot pass anything else.

```rust
# #[cfg(feature = "alloc")]
# {
use confined::{Confined, Error};

/// One to three signers: never none, never four.
type Signers = Confined<Vec<&'static str>, 1, 3>;

// The size is checked once, when the value is created...
let mut signers = Signers::try_from(vec!["alice"]) ?;
assert_eq!(Signers::try_from(vec![]), Err(Error::Undersize { len: 0, min_len: 1 }));

// ...and again by every operation that changes it.
signers.push("bob") ?;
signers.push("carol") ?;
assert_eq!(signers.push("dave"), Err(Error::Oversize { len: 3, max_len: 3 }));

signers.remove(0) ?;
signers.remove(0) ?;
assert_eq!(signers.remove(0), Err(Error::Undersize { len: 1, min_len: 1 }));

// Everything read-only comes from the inner `Vec` through `Deref`.
assert_eq!(signers.len(), 1);
assert!(signers.contains(&"carol"));

// The list cannot be empty, so `first()` returns a value, not an `Option`.
assert_eq!(*signers.first(), "carol");
# }
Ok::<(), confined::Error>(())
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
- **Bounded collection length.** An upper limit caps the number of items (or string bytes), not
  total memory or processing time; see
  [SECURITY.md](./SECURITY.md) for the resource model.
- **No memory overhead.** `Confined` is a newtype with the same size as the
  collection it wraps. The limits exist only in the type.

## What "compile-time" means here

The limits are const generic parameters, so they are fixed at compile time and
two different limits give two different types. The number of elements is only
known at run time, so the actual check happens at run time: on construction and
in every size-changing method. A violation is returned as an
[`Error`](#errors). Constructors named `*_checked` or `*_checked_in` panic
instead. Convenience constructors such as `with()` can also panic when the
underlying fixed capacity is insufficient; infallible allocation can abort
on out-of-memory conditions.

The compiler contributes three things.

1. **The unchecked mutators are out of reach.** `Confined` implements `Deref`
   but not `DerefMut`, so all read-only methods of the inner collection are
   available, while its own `push`, `insert`, `remove` and similar methods are
   not. Only the checked replacements defined on `Confined` can change the
   size.
2. **Methods are available across broader ranges of limits.** When a method
   requires a particular bound, an inline const assertion checks it when the
   program is built. The first table below lists the conditions.
3. **Sizes known at build time are checked at build time.** Where every size
   involved is a constant, a wrong combination stops the build instead of
   returning an error at run time. The second table lists these checks.

| Condition                 | Methods                                                                                                                                                                                                                                                                                                                                              |
|---------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `MIN_LEN` is `0`          | `new()`, `new_in()`, `with_capacity()`, `with_capacity_in()`, `try_with_capacity()`, `try_with_capacity_in()`, `Default`, `clear()`; `pop()` on `Vec`, `String`, `AsciiString` and `BinaryHeap`; `pop_front()` and `pop_back()` on `VecDeque` and `LinkedList`; `truncate()` and `drain()` on `VecDeque`; `retain()` on maps; `drain()` on `HashMap` |
| `MIN_LEN > 0`             | `first()` and `last()` on indexable collections (`Vec`, `VecDeque`, `AsciiString`, `IndexSet`); `first_mut()` and `last_mut()` where the collection also supports mutable indexing (`Vec`, `VecDeque`, `AsciiString`)                                                                                                                                |
| `MIN_LEN <= 1 <= MAX_LEN` | `with(item)`, `with_in(item, allocator)`; `with_key_value(k, v)` on maps                                                                                                                                                                                                                                                                             |
| `MAX_LEN <= U8MAX`        | `len_u8() -> u8`                                                                                                                                                                                                                                                                                                                                     |
| `MAX_LEN <= U16MAX`       | `len_u16() -> u16`                                                                                                                                                                                                                                                                                                                                   |
| `MAX_LEN <= U24MAX`       | `len_u24() -> amplify_num::u24`                                                                                                                                                                                                                                                                                                                      |
| `MAX_LEN <= U32MAX`       | `len_u32() -> u32`                                                                                                                                                                                                                                                                                                                                   |

These are range checks, not exact matches: for example, `MIN_LEN = 2` is
enough for `first()` and `last()`, and `len_u8()` works for any maximum no
greater than `u8::MAX`. The methods are available on the broader type range,
and the build-time assertions reject bounds that would make their result
unsafe.

| Function                                          | The build fails unless                                  |
|---------------------------------------------------|---------------------------------------------------------|
| `from_array([T; N])` on `Vec`                     | `MIN_LEN <= N` and `N <= MAX_LEN`                       |
| `widen::<MIN2, MAX2>()`                           | `MIN2 <= MIN_LEN` and `MAX_LEN <= MAX2`                 |
| `with(item)`, `with_in(item, allocator)`          | `MIN_LEN <= 1 <= MAX_LEN` and `MAX_LEN >= MAX_ITEM_LEN` |
| `with_key_value(k, v)`                            | `MIN_LEN <= 1 <= MAX_LEN`                               |
| `first()`, `last()`, `first_mut()`, `last_mut()`  | `MIN_LEN > 0`                                           |
| `len_u8()`, `len_u16()`, `len_u24()`, `len_u32()` | `MAX_LEN` does not exceed the return type's maximum     |
| `try_from` and the constructors built on it       | `MIN_LEN <= MAX_LEN`                                    |

For example, `Confined::<Vec<u8>, 1, 0>::with(7)` stops the build with the
message "the number of items is outside of the confinement bounds". These
checks run when machine code is generated: `cargo build` and `cargo test`
report them, but `cargo check` does not.

## Installation

Use the Git dependency while `confined` is unpublished:

```toml
[dependencies]
confined = { git = "https://github.com/rust-amplify/confined" }
```

The minimum supported Rust version is 1.100.0; the crate uses edition 2024.
No downstream `[patch]` is needed. The dependency imported as `collections`
is the released `collections-traits` 0.1.0 package.

### Cargo features

| Feature       |  Default  | What it adds                                                                                                                             |
|---------------|:---------:|------------------------------------------------------------------------------------------------------------------------------------------|
| `std`         |    yes    | Implies `alloc`; standard `HashMap` / `HashSet`, `std::io::Write`, and standard-library integration for enabled dependencies             |
| `alloc`       | via `std` | Allocating collections such as `Vec`, `String`, trees, lists and heaps, usable without `std`                                             |
| `hex`         |    yes    | Hex formatting and bounded `amplify_num::hex::FromHex` for confined byte vectors when `alloc` is enabled; does not itself enable `alloc` |
| `ascii`       |    no     | Implies `alloc`; `AsciiString`, `AsciiError`, and the `confined::ascii` re-export                                                        |
| `indexmap`    |    no     | Implies `alloc`, not `std`; custom-hasher `IndexMap` / `IndexSet` and the `confined::indexmap` re-export                                 |
| `hashbrown`   |    no     | Implies `alloc`, not `std`; allocator-aware `HashMap` / `HashSet` and the `confined::hashbrown` re-export                                |
| `heapless`    |    no     | Fixed-capacity collections and the `confined::heapless` re-export; no allocator required                                                 |
| `serde`       |    no     | `Serialize` / `Deserialize` when the inner collection supports them; does not itself enable `alloc` or ASCII serialization               |
| `ascii-serde` |    no     | Enables `ascii`, `serde`, `std` and `ascii/serde` for ASCII serialization                                                                |
| `all`         |    no     | Everything above, including the std-only `ascii-serde` integration                                                                       |

Default features are `std` + `hex`. With `default-features = false`, the core
wrapper needs neither `std` nor `alloc`. `heapless`, `serde` and `hex`, alone
or together, do not opt into allocation. Use `alloc` explicitly for standard
allocating collections, or enable `ascii`, `indexmap` or `hashbrown`, which
imply it. For example:

```toml
[dependencies]
confined = { git = "https://github.com/rust-amplify/confined", default-features = false, features = ["heapless"] }
```

The ready-made index aliases and constructor macros use a default hasher and
require `std` + `indexmap`. In `no_std` + `alloc`, wrap an index collection with
an explicit `S: BuildHasher` in `Confined`; `S: Default` is only needed by
constructors that create a default hasher.

**Breaking feature change:** `ascii` + `serde` now compiles without `std`, but
does not enable serialization of ASCII collections. Use `ascii-serde` for
that integration. It requires `std` because ASCII 1.1's serde implementation
unconditionally imports `std`.

## Usage

### Creating and releasing

```rust
# #[cfg(feature = "alloc")]
# {
use confined::Confined;

type UpToThree = Confined<Vec<u8>, 1, 3>;

let a = UpToThree::try_from(vec![1, 2]) ?; // from an owned collection
let b = UpToThree::try_from_iter([1, 2]) ?; // from an iterator
let c = UpToThree::try_from_slice(& [1, 2]) ?; // `Vec` only: clones a slice
let d = UpToThree::from_checked(vec![1, 2]); // panics instead of returning `Err`
let e = UpToThree::from_array([1, 2]); // `Vec` only: the length is checked by the build
assert!(a == b && b == c && c == d && d == e);

// A lower limit of 0 or 1 unlocks infallible constructors.
let empty = Confined::<Vec<u8>, 0, 3 >::new();
let single = UpToThree::with(7);
assert!(empty.is_empty() && single.len() == 1);

// Moving to wider limits cannot fail, so it is not checked at run time.
let wide: Confined<Vec<u8>, 0, 255 > = single.widen();
assert_eq!(wide.len_u8(), 1);

// Getting the plain collection back.
let borrowed: & Vec<u8> = a.as_unconfined();
assert_eq!(borrowed.len(), 2);
let cloned: Vec<u8> = a.to_unconfined();
let owned: Vec<u8> = a.release();
assert_eq!(cloned, owned);

# }
Ok::<(), confined::Error>(())
```

`try_from_iter` builds lazily, ignoring iterator size hints and without
reserving `MIN_LEN` in advance. It stops at the first item that would exceed
the bound or native capacity. `Oversize` describes the rejected growth, not
the iterator's total length. An infinite iterator of duplicates into a set
or map need never grow, so iteration is **not** guaranteed to terminate.
Borrowed slices, UTF-8 strings and ASCII strings are length-checked before
copying into owned storage.

`Confined::try_from` is an inherent function that takes the collection itself. (It cannot be a
`TryFrom` impl because of the blanket impl in `core`.) Strings
additionally implement `FromStr` and the real `TryFrom<&str>` trait. The
inherent function shadows the latter, so use `parse()` or `try_into()` for
string slices:

```rust
# #[cfg(feature = "alloc")]
# {
use confined::{Confined, Error};

type Name = Confined<String, 1, 32>;

let name: Name = "satoshi".parse() ?;
assert_eq!(name.to_string(), "satoshi");
let same: Name = "satoshi".try_into() ?;
assert_eq!(name, same);

assert_eq!("".parse::<Name>(), Err(Error::Undersize { len: 0, min_len: 1 }));

# }
Ok::<(), confined::Error>(())
```

### Allocator-aware vectors

The same `Confined` type wraps `Vec<T, A>` for any `A: Allocator`; there is no
separate allocator-specific wrapper. Pass the allocator explicitly with
`new_in(allocator)`, `with_capacity_in(capacity, allocator)`,
`try_with_capacity_in(capacity, allocator)`,
`with_in(item, allocator)`, `try_from_iter_in(iter, allocator)` or
`from_iter_checked_in(iter, allocator)`. These constructors do not require
`A: Default`, including `with_in`.

```rust
# #[cfg(feature = "std")]
# {
use std::alloc::System;

use confined::{Confined, Error};

type Packet = Confined<Vec<u8, System>, 1, 4>;

let mut packet = Packet::with_in(0xCA, System);
packet.push(0xFE) ?;
let same = Packet::try_from_iter_in([0xCA, 0xFE], System) ?;
assert_eq!(packet.as_slice(), same.as_slice());

// Releasing the confinement preserves the vector's allocator.
let bytes: Vec<u8, System> = packet.release();
assert_eq!(bytes.as_slice(), &[0xCA, 0xFE]);

# }
Ok::<(), confined::Error>(())
```

`new_in`, `with_capacity_in` and `try_with_capacity_in` require `MIN_LEN = 0`;
`with_in` requires bounds that can hold one item. Both iterator constructors
stop on rejected growth; `from_iter_checked_in` panics on an error.
Vector and blob aliases also accept an allocator, defaulting to `Global`,
for example `TinyVec<T, A = Global>` and `TinyBlob<A = Global>`.

### Fallible reservation

On supported reservable collections, `try_reserve(additional)` first rejects
requests larger than the remaining confinement allowance, then uses fallible
native reservation. Native capacity or allocation failure is reported as
`Error::CapacityExceeded { len }`. `try_with_capacity(cap)` and
`try_with_capacity_in(cap, allocator)` cap the initial reservation request at
`MAX_LEN`. Native allocators may round capacity up; confinement limits length,
not the number of allocated bytes.

```rust
# #[cfg(feature = "alloc")]
# {
use confined::Confined;

let mut bytes = Confined::<Vec<u8>, 0, 4 >::try_with_capacity(100) ?;
bytes.extend([1, 2]) ?;
bytes.try_reserve(2) ?;
assert!(bytes.try_reserve(3).is_err()); // only two more items fit
# }
Ok::<(), confined::Error>(())
```

### Modifying

```rust
# #[cfg(feature = "alloc")]
# {
use confined::{Confined, Error};

let mut v = Confined::<Vec<u8>, 0, 3 >::new();
v.push(1) ?;
v.extend([2, 3]) ?;
assert_eq!(v.push(4), Err(Error::Oversize { len: 3, max_len: 3 }));

// Changing items in place cannot break the limits, so it is unrestricted.
v[0] = 10;
for item in & mut v {
* item += 1;
}
assert_eq!(v.as_slice(), &[11, 3, 4]);

assert_eq!(v.remove(7), Err(Error::OutOfBoundary { index: 7, len: 3 }));
assert_eq!(v.pop(), Some(4)); // exists only because `MIN_LEN` is 0
v.clear(); // same

// A list that is never empty gives direct access to both of its ends.
let mut v = Confined::<Vec<u8>, 1, 3 >::from_array([1, 2]);
* v.first_mut() += 10;
* v.last_mut() += 20;
assert_eq!(v.as_slice(), &[11, 22]);

# }
Ok::<(), confined::Error>(())
```

### Maps and sets

```rust
# #[cfg(feature = "alloc")]
# {
# extern crate alloc;
use alloc::collections::BTreeMap;

use confined::{Confined, Error};

let mut votes = Confined::<BTreeMap< & str, u32>, 1, 2 >::with_key_value("alice", 1);
votes.insert("bob", 1) ?;

// Overwriting an existing key does not grow the map, so it works when full.
assert_eq!(votes.insert("alice", 5)?, Some(1));
assert_eq!(votes.insert("carol", 1), Err(Error::Oversize { len: 2, max_len: 2 }));

*votes.get_mut( & "bob").unwrap() += 1;
assert_eq!(votes[&"bob"], 2);

// `entry()` checks the bound and prepares native room before returning.
*votes.entry("bob") ?.or_insert(0) += 1;
assert_eq!(votes[&"bob"], 3);
assert!(votes.entry("carol").is_err()); // a new key, and the map is full

// Removing a missing key is not an error. Removing the last allowed entry is.
assert_eq!(votes.remove(&"dave")?, None);
assert_eq!(votes.remove(&"bob")?, Some(3));
assert_eq!(votes.remove(&"alice"), Err(Error::Undersize { len: 1, min_len: 1 }));

# }
Ok::<(), confined::Error>(())
```

`entry()` returns `confined::Entry`, not the entry type of the wrapped map. It
has `key`, `or_insert`, `or_insert_with`, `or_default` and `and_modify` for
maps implementing `collections::CollectionEntry`, including your own maps
whose entry types implement `collections::MapEntry`. `or_insert_with_key`
is additionally available for the built-in `BTreeMap`, `HashMap` and
`IndexMap`. Nothing on `Entry` removes the entry: removal goes through
`remove` on the confined map, which checks the lower limit.

`CollectionMap` does not require an entry API. Its generic `insert`, `get_mut`,
`values_mut` and retention methods also work for entry-less maps. Entry
insertion prepares native room before exposing an infallible entry operation
where the collection supports fallible capacity handling; tree allocation
can still abort on out-of-memory conditions.

Sets use the generic `push` to add an item and `remove` / `take` to delete one.
There are no confined set `insert` or `retain` methods; those names are used
by the generic map API. `heapless::IndexSet` has a direct checked
`remove(&T) -> Result<bool, Error>`, but no `take`, because upstream does not
implement `CollectionSet` for it. Duplicate set items and existing map keys
do not grow the collection, so `push` and `extend` accept them even at
`MAX_LEN`.

### Deques, linked lists and heaps

Use a deque or linked list for ordered work, or a max-heap for priorities:

```rust
# #[cfg(feature = "alloc")]
# {
use confined::{tiny_deque, tiny_heap, tiny_list, Error};

let mut queue = tiny_deque!["queued"];
queue.push_front("urgent") ?;
assert_eq!(queue.pop_front(), Some("urgent"));

let mut steps = tiny_list!["parse", "save"];
steps.push_front("read") ?;
assert_eq!(steps.remove(1)?, "parse");
assert_eq!(steps.pop_back(), Some("save"));

let mut priorities = tiny_heap![1, 3, 2];
priorities.push(5) ?;
assert_eq!(priorities.peek(), Some(&5));
assert_eq!(priorities.pop(), Some(5));

# }
Ok::<(), confined::Error>(())
```

`LinkedList::remove(index)` checks the lower bound. List `pop_front` /
`pop_back` and heap `pop` are available only with `MIN_LEN = 0`, so they
cannot empty a non-empty confinement. Heaps require `T: Ord`.

### Fixed-capacity collections without an allocator

Enable `heapless` with default features disabled for allocator-free
`heapless::Vec<T, N, L>`, `String<N, L>`, `Deque`, `BinaryHeap`, `IndexSet`,
`IndexMap` and `LinearMap`. Both the confinement bounds and native capacity
apply. Checked `push`, `extend`, `insert` and `entry` operations report
`Error::CapacityExceeded { len }` instead of panicking when native capacity
runs out, even if `MAX_LEN` would allow another item.

```rust
# #[cfg(feature = "heapless")]
# {
use confined::{Error, HeaplessVec};

let mut samples = HeaplessVec::<u8, 2, 0, 3 >::new();
samples.extend([10, 20]) ?;
assert!(matches!(samples.push(30), Err(Error::CapacityExceeded { .. })));
assert_eq!(samples.len(), 2);
# }
Ok::<(), confined::Error>(())
```

The aliases keep native capacity separate from confinement limits:

| Alias               | Parameters                                       |
|---------------------|--------------------------------------------------|
| `HeaplessVec`       | `<T, CAP, MIN = 0, MAX = usize::MAX, L = usize>` |
| `HeaplessString`    | `<CAP, MIN = 0, MAX = usize::MAX, L = usize>`    |
| `HeaplessDeque`     | `<T, CAP, MIN = 0, MAX = usize::MAX>`            |
| `HeaplessHeap`      | `<T, K, CAP, MIN = 0, MAX = usize::MAX>`         |
| `HeaplessIndexSet`  | `<T, S, CAP, MIN = 0, MAX = usize::MAX>`         |
| `HeaplessIndexMap`  | `<K, V, S, CAP, MIN = 0, MAX = usize::MAX>`      |
| `HeaplessLinearMap` | `<K, V, CAP, MIN = 0, MAX = usize::MAX>`         |

`K` in `HeaplessHeap` selects the heap kind; `S` is the index collection's
hasher. Convenience constructors such as `new()` and `with()` follow the
underlying constructor's behavior; `with()` can panic if native capacity is
too small. Use checked construction and mutation for fallible input paths.

### Size constants, aliases and macros

The crate exports constants for the common limits:

| Constant | Value                      |
|----------|----------------------------|
| `ZERO`   | `0`                        |
| `ONE`    | `1`                        |
| `U8MAX`  | `u8::MAX` (255)            |
| `U16MAX` | `u16::MAX` (65 535)        |
| `U24MAX` | min(2²⁴ − 1, `usize::MAX`) |
| `U32MAX` | min(2³² − 1, `usize::MAX`) |
| `U40MAX` | min(2⁴⁰ − 1, `usize::MAX`) |
| `U48MAX` | min(2⁴⁸ − 1, `usize::MAX`) |
| `U56MAX` | min(2⁵⁶ − 1, `usize::MAX`) |
| `U64MAX` | min(2⁶⁴ − 1, `usize::MAX`) |

The `len_u8()` family accepts any maximum no greater than the corresponding
constant; the ready-made aliases use these constants as their exact maxima.
Wide constants are capped, not truncated, on narrower targets; conversions
to `amplify_num` integers likewise avoid truncating intermediate casts.
The name of an alias is a size prefix
followed by a collection suffix, for example `TinyVec<T>` or
`NonEmptyOrdMap<K, V>`:

| Prefix     | Limits                                                           |
|------------|------------------------------------------------------------------|
| `Tiny`     | `0..=U8MAX`                                                      |
| `Small`    | `0..=U16MAX`                                                     |
| `Medium`   | `0..=U24MAX`                                                     |
| `Large`    | `0..=U32MAX`                                                     |
| `NonEmpty` | `1..=MAX`, where `MAX` is a parameter with the default `U64MAX`  |
| `Confined` | `MIN..=MAX`, both parameters, with the defaults `0` and `U64MAX` |

| Suffix                 | Collection                      | Feature            |
|------------------------|---------------------------------|--------------------|
| `String`               | `String`                        | `alloc`            |
| `Ascii`                | `AsciiString`                   | `ascii`            |
| `Blob`                 | `Vec<u8, A>` (`A = Global`)     | `alloc`            |
| `Vec`                  | `Vec<T, A>` (`A = Global`)      | `alloc`            |
| `Deque`                | `VecDeque<T>`                   | `alloc`            |
| `List`                 | `LinkedList<T>`                 | `alloc`            |
| `Heap`                 | `BinaryHeap<T>`                 | `alloc`            |
| `OrdSet`, `OrdMap`     | `BTreeSet<T>`, `BTreeMap<K, V>` | `alloc`            |
| `HashSet`, `HashMap`   | `HashSet<T>`, `HashMap<K, V>`   | `std`              |
| `IndexSet`, `IndexMap` | `IndexSet<T>`, `IndexMap<K, V>` | `std` + `indexmap` |

`confined::SmallVec<T>` is therefore a `Vec` of up to 65 535 items. It is not
related to the `smallvec` crate. Every prefix is available for both `List`
and `Heap`, including `LargeList`, `LargeHeap`, `NonEmptyList` and
`NonEmptyHeap`. Hash and index aliases keep the default hasher; wrap a
custom-hasher collection directly in `Confined` instead.

```rust
# #[cfg(feature = "alloc")]
# {
use confined::{NonEmptyVec, SmallString, TinyBlob};

/// Writes a blob prefixed with its length as a single byte.
fn encode(blob: &TinyBlob, out: &mut Vec<u8>) {
    out.push(blob.len_u8()); // the type guarantees that the length fits
    out.extend_from_slice(blob.as_slice());
}

let blob = TinyBlob::try_from(vec![0xCA, 0xFE]) ?;
let mut out = vec![];
encode( & blob, & mut out);
assert_eq!(out, [2, 0xCA, 0xFE]);

let text = SmallString::try_from("hello".to_owned()) ?;
assert_eq!(text.len_u16(), 5);

let items = NonEmptyVec::<u8, 3 >::try_from_iter([1, 2, 3]) ?;
assert_eq!(*items.last(), 3);

# }
Ok::<(), confined::Error>(())
```

An alias of your own takes one line, for example
`type Signers = Confined<Vec<PublicKey>, 1, 15>;`.

The `Tiny`, `Small` and `Medium` aliases have constructor macros. Their names
start with `tiny_`, `small_` or `medium_`:

| Macro suffix     | Builds                             | Feature            |
|------------------|------------------------------------|--------------------|
| `s!`             | a `String` from a string literal   | `alloc`            |
| `blob!`          | a `Vec<u8>`                        | `alloc`            |
| `vec!`           | a `Vec<T>`                         | `alloc`            |
| `deque!`         | a `VecDeque<T>`                    | `alloc`            |
| `list!`          | a `LinkedList<T>`                  | `alloc`            |
| `heap!`          | a `BinaryHeap<T>`                  | `alloc`            |
| `bset!`, `bmap!` | a `BTreeSet<T>`, `BTreeMap<K, V>`  | `alloc`            |
| `set!`, `map!`   | a `HashSet<T>`, `HashMap<K, V>`    | `std`              |
| `iset!`, `imap!` | an `IndexSet<T>`, `IndexMap<K, V>` | `std` + `indexmap` |

```rust
# #[cfg(feature = "alloc")]
# {
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
# }
```

A macro panics if its content does not fit the limits of the type it builds.

### Byte strings

With the default `std` and `hex` features a confined `Vec<u8>` can be used as a
bounded write buffer, printed as hex and parsed from hex:

```rust
# #[cfg(all(feature = "std", feature = "hex"))]
# {
use std::io::Write;

use amplify_num::hex::{self, FromHex};
use confined::Confined;

let mut blob = Confined::<Vec<u8>, 0, 4 >::new();
blob.write_all(& [0xDE, 0xAD, 0xBE, 0xEF]) ?;
assert!(blob.write_all(&[0x00]).is_err()); // does not fit, nothing is written
assert_eq!(format!("{blob:x}"), "deadbeef");
assert_eq!(format!("{blob:X}"), "DEADBEEF");

assert_eq!(Confined::<Vec<u8>, 0, 4>::from_hex("deadbeef"), Ok(blob));
assert_eq!(
    Confined::<Vec<u8>, 0, 3>::from_hex("deadbeef"),
    Err(hex::Error::InvalidLength(3, 4)) // the limit and the actual length
);

# }
Ok::<(), std::io::Error>(())
```

A `write` call either stores the whole buffer or fails with
`io::ErrorKind::OutOfMemory`. The `FromHex` trait and its error type come from
the [`amplify_num`](https://crates.io/crates/amplify_num) crate.
Writing and hex formatting also work for `Vec<u8, A>` with any allocator;
`FromHex` additionally requires `A: Default` to create the vector.
Checked hex decoding enforces the bound without allocating from untrusted
iterator size hints. Hex formatting streams to the formatter without an
intermediate allocation; an output consumer such as `format!` may allocate.

### ASCII strings

With the `ascii` feature:

```rust
# #[cfg(feature = "ascii")]
# {
use confined::ascii::AsciiString;
use confined::{AsciiError, Confined, Error};

type Ticker = Confined<AsciiString, 1, 8>;

let ticker: Ticker = "BTC".parse() ?;
assert_eq!(ticker.len(), 3);
assert!(Ticker::try_from(AsciiString::new()).is_err()); // too short

// A string slice can fail in two ways.
assert!(matches!("₿".parse::<Ticker>(), Err(AsciiError::Ascii(_))));
assert_eq!(
    "BITCOIN-USD".parse::<Ticker>(),
    Err(AsciiError::Confinement(Error::Oversize { len: 11, max_len: 8 }))
);

# }
Ok::<(), Box<dyn std::error::Error> > (())
```

### Errors

```rust,ignore
pub enum Error {
    /// The operation would leave fewer than `MIN_LEN` items.
    Undersize { len: usize, min_len: usize },
    /// The operation would leave more than `MAX_LEN` items.
    Oversize { len: usize, max_len: usize },
    /// The underlying collection cannot provide the requested capacity.
    CapacityExceeded { len: usize },
    /// Index-based removal with an index past the end.
    OutOfBoundary { index: usize, len: usize },
}
```

`Error` is `Copy` and implements `Display` and `core::error::Error`, even in
`no_std`. `AsciiError` also implements `core::error::Error` whenever the
`ascii` feature is enabled; neither error implementation requires `std`.

### Serde

With `serde`, `Confined` implements `Serialize` and `Deserialize` when the
inner collection supports them. It serializes as a newtype, so formats like
JSON see the inner collection. Deserialization checks the limits **after
constructing the inner collection**: invalid lengths are rejected with a
custom error carrying the matching [`Error`](#errors), but construction may
already have allocated or processed untrusted data. Configure resource limits
in the deserializer; this is not a bounded streaming parser.

`serde` alone does not enable `alloc`. ASCII serialization requires the
separate, std-only `ascii-serde` feature, not just `ascii` + `serde`.

## Supported collections

`Confined<C, MIN_LEN, MAX_LEN>` has no struct-level collection bound. Its
size-aware generic methods require `ItemLen: collections::CollectionBase`.
The [`collections-traits`](https://github.com/rust-amplify/collections)
package is imported as `collections`: `CollectionBase` owns `Item`,
`item_count`, `push` and `clear`; `Collection` adds `with_capacity` and the
`Default` / `Extend` / `FromIterator` construction requirements.
`CollectionMap` and `CollectionEntry` are separate, so map mutation does not
require entry support. Sets use `CollectionSet` where available.

| Collection                                                      | Feature     | Checked methods specific to it                                                                                       |
|-----------------------------------------------------------------|-------------|----------------------------------------------------------------------------------------------------------------------|
| `Vec<T, A>`                                                     | `alloc`     | `as_slice`, `into_vec`, `iter`, `remove`, `pop`¹; `from_array`, `try_from_slice`, `from_slice_checked` with `Global` |
| `Vec<u8, A>`                                                    | `alloc`     | `std::io::Write` (`std`); `LowerHex`, `UpperHex` (`hex`); `FromHex` (`hex`, `A: Default`)                            |
| `String`                                                        | `alloc`     | `TryFrom<&str>`, `FromStr`, `remove`, `pop`¹                                                                         |
| `AsciiString`                                                   | `ascii`     | `TryFrom<&str>`, `FromStr`, `remove`, `pop`¹                                                                         |
| `VecDeque<T>`                                                   | `alloc`     | `push_front`, `push_back`, `remove`, `pop_front`¹, `pop_back`¹, `drain`¹, `truncate`¹                                |
| `LinkedList<T>`                                                 | `alloc`     | `push_front`, `push_back`, `remove`, `pop_front`¹, `pop_back`¹                                                       |
| `BinaryHeap<T>`                                                 | `alloc`     | `pop`¹; generic `push` / `extend` with `T: Ord`                                                                      |
| `BTreeSet<T>`                                                   | `alloc`     | `remove`, `take`                                                                                                     |
| `HashSet<T, S>`                                                 | `std`       | `remove`, `take`                                                                                                     |
| `IndexSet<T, S>`                                                | `indexmap`  | `remove`, `take`                                                                                                     |
| `BTreeMap<K, V>`                                                | `alloc`     | `remove`, `remove_entry`, `pop_first`, `pop_last`, `append`, `split_off`, `range_mut`, `into_keys`, `into_values`    |
| `HashMap<K, V, S>`                                              | `std`       | `remove`, `remove_entry`, `reserve`, `shrink_to_fit`, `shrink_to`, `into_keys`, `into_values`, `drain`¹              |
| `IndexMap<K, V, S>`                                             | `indexmap`  | `remove`, `into_keys`, `into_values`                                                                                 |
| `hashbrown::HashMap<K, V, S, A>`, `HashSet<T, S, A>`            | `hashbrown` | Custom hashers and allocators; fallible native capacity handling                                                     |
| `heapless::Vec<T, N, L>`, `String<N, L>`, `Deque`, `BinaryHeap` | `heapless`  | Fixed-capacity checked mutation without allocation                                                                   |
| `heapless::IndexSet`, `IndexMap`, `LinearMap`                   | `heapless`  | Fixed-capacity set/map operations; `IndexSet::remove` but no `take`                                                  |

¹ Only when `MIN_LEN` is `0`. `std::io::Write` is available for any `MIN_LEN`,
because writing only appends bytes.

`BTreeMap` keys need `Ord`, not `Hash`. `HashMap`, `HashSet`, `IndexMap` and
`IndexSet` support custom hashers `S: BuildHasher`; `Default` is required
only by constructors that need it. `hashbrown` maps and sets additionally
support `A: Allocator`, without requiring `A: Default` for native operations.
Custom-hasher `indexmap` collections work in `no_std` + `alloc`; their
default-hasher aliases and macros require `std`.

**Hostile keys:** hashbrown's default hasher is not collision-DoS resistant.
Choose a keyed/randomized hasher suitable for adversarial input; bounding the
number of keys does not make hashing constant-time.

For an ordinary custom `CollectionBase`, an empty
`impl confined::ItemLen for MyCollection {}` is sufficient. `Item` is inherited
from `CollectionBase`; there is no `Access`, `ViaCollection` or
`ViaCollectionAlloc` dispatch layer. Its hooks are:

| Hook                                            | Default / contract                                                                           |
|-------------------------------------------------|----------------------------------------------------------------------------------------------|
| `confined_len(&self) -> usize`                  | Uses `item_count`; override when confinement units differ                                    |
| `item_len(&Item) -> usize`                      | Returns `1`                                                                                  |
| `MAX_ITEM_LEN`                                  | `1`; must bound the length contribution of one item                                          |
| `item_growth(&self, &Item) -> usize`            | Uses `item_len`; sets/maps return `0` for duplicates                                         |
| `try_push(&mut self, Item) -> Result<(), Item>` | Uses infallible `CollectionBase::push`; capacity-backed implementations override it fallibly |
| `prepare_entry(&mut self) -> Result<(), Error>` | Ensures native room before exposing an infallible entry insertion                            |

Custom implementations must keep length, growth and mutation contracts
consistent. UTF-8 strings override the character-counting upstream
`item_count`: confinement uses `CollectionBytes` for O(1) byte length and
counts each inserted character by its encoded byte length.

The generic API is available with `ItemLen` and the relevant operation's
additional bounds (for example `CollectionMap`, `Index` or `Clone`):

| Purpose               | Items                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------|
| Create                | `try_from`, `try_from_iter`, `from_checked`, `from_iter_checked`                                                                                     |
| Create with allocator | `new_in`, `with_capacity_in`, `try_with_capacity_in`, `with_in`, `try_from_iter_in`, `from_iter_checked_in` on supported allocator-aware collections |
| Reserve               | `try_reserve`, `try_with_capacity`, `try_with_capacity_in` on supported reservable collections                                                       |
| Change limits         | `widen` to limits that include the current ones                                                                                                      |
| Read                  | `Deref<Target = C>`, `as_unconfined`, `to_unconfined`, `AsRef<[T]>`, `Borrow<[T]>`, `Index`, `Display`, iteration over `&Confined`                   |
| Change items          | `IndexMut`, `AsMut<[T]>`, `BorrowMut<[T]>`, `iter_mut`, iteration over `&mut Confined`                                                               |
| Change size           | `push`, `extend`                                                                                                                                     |
| Unwrap                | `release`, by-value `IntoIterator`                                                                                                                   |
| Maps                  | `insert`, `get_mut`, `values_mut`, `keyed_values_mut`, `try_retain`, `retain`¹; `entry` additionally requires `CollectionEntry`                      |
| Re-validate           | `with_mut(f)` returns `Err` and `as_mut_checked(f)` panics if the limits do not hold after the closure has run                                       |
| Derived traits        | `Clone`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, whenever the inner collection implements them                                      |

## Behaviour worth knowing

- **UTF-8 confinement measures bytes in O(1), not characters.** For `String`,
  `"añb"` has three characters but needs `MAX_LEN >= 4`.
  `push` and `remove` count the bytes of the character they add or take away,
  so `push('€')` needs three free bytes. `with()` needs `MAX_LEN >= 4` for a
  `String`, whatever the character is, because the check is made at build time
  and a character may take `char::MAX_LEN_UTF8` (four) bytes.
- **`extend` is not atomic.** It pushes items one by one and stops at the first
  one that does not fit. The items before it stay in the collection.
- **Duplicates do not consume length.** `push` and `extend` on sets and maps
  accept duplicate items or existing keys even at `MAX_LEN`. Likewise,
  replacing a value with `insert` or accessing an occupied `entry` does not
  require room for another entry.
- **Removing something absent is not an error.** `remove` and `take` on maps and
  sets return `Ok(None)` or `Ok(false)`, even at the minimum size.
- **`String::remove` needs a character boundary.** An index inside a multi-byte
  character returns `OutOfBoundary` instead of panicking.
- **Non-empty accessors follow the lower bound.** `first()`, `last()`,
  `first_mut()` and `last_mut()` require `MIN_LEN > 0`, checked at build time.
  On indexable wrapped collections, `first()` and `last()` shadow the inner
  `Option`-returning methods even when `MIN_LEN = 0`; use `as_slice().first()` /
  `as_slice().last()` for a `Vec`, or `as_unconfined()` for the wrapped
  collection's optional result.
  `BTreeSet`'s own `first()` and `last()` are unaffected because it is not
  indexable by position.
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
- **`BTreeMap::split_off` clones before mutation.** It requires `K: Clone` and
  `V: Clone`, and returns cloned entries. This avoids Rust 1.100's panic-unsafe
  native split: comparisons and cloning finish before removing originals.
  The additional memory cost and panic guarantees are explained in
  [SECURITY.md](./SECURITY.md).
- **`HashMap::reserve` is capped.** It reserves room for at most
  `MAX_LEN - len` more entries, whatever amount is asked for.
- **Fallible reservation is stricter.** `try_reserve` rejects an additional
  count beyond the remaining confinement allowance before asking the native
  collection for room. Capacity-backed checked growth reports native failure
  as `CapacityExceeded`, including fixed-capacity heapless storage.
- **A bound is not a resource budget.** Nested item sizes, hashing, iterator
  work and deserialization need their own limits; see [SECURITY.md](./SECURITY.md).

## Status and known limitations

- **ASCII serialization needs `std`.** `ascii` + `serde` is supported without
  `std`, but ASCII serialization explicitly requires `ascii-serde` because
  ASCII 1.1's serde code imports `std` unconditionally.
- **ASCII requires pointer atomics.** ASCII 1.1 imports `alloc::sync::Arc`
  unconditionally, so its optional integration cannot build on Cortex-M0 (`thumbv6m-none-eabi`).
  Core, heapless, and the other allocating integrations
  work there; ASCII is checked on Cortex-M4 (`thumbv7em-none-eabi`).
- **Not all allocation is fallible.** Trees, linked lists and ASCII strings
  lack upstream fallible reservation and can abort on OOM. `Clone`, `From`,
  `Default` and explicit infallible constructors retain underlying behavior.
  Fixed-capacity convenience construction can panic when capacity is too small.
  `FromHex` bounds decoding but retains infallible allocation because its
  upstream error type cannot represent allocation failure.
- **Deserialization validates after construction.** The deserializer must
  impose its own resource limits on untrusted input.
- **Iteration has no general time bound.** Duplicate-only or otherwise
  non-growing valid iterators may never exhaust the length allowance.
- **Custom traits are a trust boundary.** Collection implementations must
  accurately report length and growth and honor their mutation contracts.
- **Set `insert` and `retain` methods are excluded.** They overlap with the
  generic map inherent methods. Sets support `push` and `remove`; `take`
  requires `CollectionSet` and is not provided for `heapless::IndexSet`.
- **`smallvec` is not supported.** The `amplify` version of this code had
  aliases and macros for `SmallVec`; the `collections` crate has no
  implementation for it.

## Repository layout

```text
src/                  wrapper, traits, aliases, macros and collection integrations
tests/                API, allocator, collection and confinement regressions
examples/cortex_m.rs  allocator-free Cortex-M startup/link smoke example
.github/workflows/    build and validation workflows
```

The examples in this file are documentation tests. Feature-dependent examples
are gated so both all-feature and no-default-feature rustdoc runs can compile.
The library forbids unsafe code with `#![forbid(unsafe_code)]`; this does not
make a claim about unsafe code in dependencies.

## Development

```console
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo test --doc --all-features
cargo test --doc --no-default-features
cargo test --workspace --lib --tests --no-default-features --features heapless,serde,hex
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo +nightly fmt --all
cargo doc --all-features --open
```

Formatting needs a nightly toolchain because `.rustfmt.toml` uses unstable
options. The build workflow covers individual features, allocator-free
feature combinations, Linux/macOS/Windows on x86-64 and ARM, and stable,
beta, nightly and MSRV 1.100.0. Its Cortex-M matrix checks
`thumbv6m-none-eabi` and `thumbv7em-none-eabi`, including `heapless` without
allocation and `alloc,hex,indexmap,hashbrown,ascii,serde` without `std`.

After installing those targets with `rustup target add`, the allocator-free
link smoke commands are:

```console
cargo rustc --example cortex_m --target thumbv6m-none-eabi --no-default-features --features heapless -- -C panic=abort -C link-arg=-e_start
cargo rustc --example cortex_m --target thumbv7em-none-eabi --no-default-features --features heapless -- -C panic=abort -C link-arg=-e_start
```

These commands test linking a no-allocator startup, not execution on hardware.
The allocating Cortex-M checks do not supply a runtime allocator or prove
that an application will fit a device's memory.

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
