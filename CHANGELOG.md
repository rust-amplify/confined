Change Log
==========

0.5.0
-----

This version closes the paths by which a confined collection could get outside
of the limits of its type, and restores the type aliases, macros and tests
which were left behind when the crate was split out of `amplify`. Some of the
fixes change the public API.

### Breaking changes

| Change | Migration |
|--------|-----------|
| `VecDeque`: `pop_front`, `pop_back`, `truncate` and `drain` exist only when `MIN_LEN` is 0, like `pop` of `Vec`. | Use `remove(index)`, which checks the lower bound. |
| `entry()` returns `confined::Entry` instead of the entry of the wrapped map. The new type can't remove the entry. | Use `or_insert`, `or_insert_with`, `or_insert_with_key`, `or_default` and `and_modify` of the entry. Remove with `remove(&key)` of the confined map. |
| The blanket `FromStr` implementation is replaced with implementations for `String` and `AsciiString`. `FromStr::Err` is `Error` for `String` (it was `Infallible`) and `AsciiError` for `AsciiString` (it was `AsAsciiStrError`). | Handle the error. |
| `"...".parse()`, `FromHex` and `Deserialize` reject the data which do not fit the limits. `FromHex` reports `hex::Error::InvalidLength` with the violated limit and the number of bytes. | None. The data which were accepted in violation of the limits are an error now. |
| `push`, `extend` and `with` require `C: ItemLen`. | Add the bound to generic code. Add `impl ItemLen for MyCollection {}` to a collection of your own. |
| `with()` fails to build if `MAX_LEN < 1`, or `MAX_LEN < 4` for `String`. `with_key_value()` fails to build if `MAX_LEN < 1`. | For a `String` with a smaller limit use `"a".parse()` or `try_from`. |
| `try_from` and the constructors built on it fail to build for a type with `MIN_LEN > MAX_LEN`. | Fix the limits. Such a type could never hold a value. |
| `BTreeMap::append` takes a source map with `MIN_LEN = 0` and has one const parameter instead of two. | Release a source which has a lower bound, and confine it again with `MIN_LEN = 0`. |
| The deprecated methods are removed. | `from_collection_unsafe` → `from_checked`, `from_iter_unsafe` → `from_iter_checked`, `from_slice_unsafe` → `from_slice_checked`, `as_inner` → `as_unconfined`, `to_inner` → `to_unconfined`, `into_inner` and `unbox` → `release`, `push_from` → `push_front`. |
| The `indexmap` feature enables `std`. | None. The feature did not build without `std`. |

The build failures listed above are reported by `cargo build` and `cargo test`,
but not by `cargo check`.

### Fixed

- `"...".parse()`, `FromHex` and serde's `Deserialize` returned values outside
  the limits of their type.
- `String`: `push` and `remove` counted a character as a single item, while
  the length of a string is measured in bytes. A multi-byte character could
  take the string above `MAX_LEN` or below `MIN_LEN`.
- `VecDeque`: `pop_front`, `pop_back`, `truncate` and `drain` ignored
  `MIN_LEN`.
- `with()` and `with_key_value()` ignored `MAX_LEN`.
- `entry()` returned the entry of the wrapped map, which could remove itself
  and take a `BTreeMap`, `HashMap` or `IndexMap` below `MIN_LEN`.
- `try_retain` returned `Undersize` after the elements were already removed,
  leaving the map below `MIN_LEN`.
- `BTreeMap::append` emptied the source map whatever its `MIN_LEN` was, and
  refused a merge which fits when the maps shared keys.
- `HashMap::reserve` passed the intended total length to the wrapped map as
  the number of additional elements. It reserved too much, and in a debug
  build `reserve(usize::MAX)` panicked on an arithmetic overflow.
- `from_slice_checked` panicked for a slice of exactly `MIN_LEN` items.
- `try_from_iter` and `from_iter_checked` read the whole iterator before
  rejecting it.
- With the `serde` feature alone, `Serialize` and `Deserialize` were not
  implemented for the confined `Vec`, `String` and other standard collections,
  because serde's `alloc` feature was not enabled.
- Builds without the `std` feature: `--no-default-features` alone and with
  `hex`, `serde` or `ascii`.
- The build with Rust 1.75.0, the declared MSRV: `Cargo.lock` pinned
  `indexmap` 2.14.2, which that toolchain can't read. It pins 2.11.4 now.
- Unresolved documentation links to `u24::MAX` and `mem::forget`.

### Added

- Type aliases for the common limits, exported from the crate root: `Tiny*`,
  `Small*`, `Medium*`, `Large*`, `Confined*` and `NonEmpty*` for `String`,
  `AsciiString` (`*Ascii`), `Vec<u8>` (`*Blob`), `Vec`, `VecDeque` (`*Deque`),
  `HashSet`, `BTreeSet` (`*OrdSet`), `HashMap`, `BTreeMap` (`*OrdMap`),
  `IndexSet` and `IndexMap`. `ConfinedHashMap` wraps `HashMap<K, V>`; the
  unported version wrapped `HashSet<K, V>`.
- Constructor macros `tiny_*!`, `small_*!` and `medium_*!` for `s`, `blob`,
  `vec`, `set`, `bset`, `map`, `bmap`, `iset` and `imap`. They do not need any
  other macro or import in the scope of the caller.
- `Entry`, the type returned by `entry()` of a confined map, with the methods
  `key`, `or_insert`, `or_insert_with`, `or_insert_with_key`, `or_default` and
  `and_modify` for `BTreeMap`, `HashMap` and `IndexMap`.
- `ItemLen` trait, which tells how much a single item adds to the length of a
  collection: one for all supported collections except `String`, where it is
  the number of bytes in the character.
- `FromStr` for confined `String` and `AsciiString`.
- `AsciiError` is exported from the crate root under the `ascii` feature.
- `first_mut()` and `last_mut()` for the collections which contain at least
  one item and can be indexed mutably by position.
- `from_array()` for `Vec`. The length of the array is checked against the
  limits when the program is built.
- `widen()`, which converts a confinement into a confinement with wider
  limits. The limits are compared when the program is built.
- Tests of the public API in `tests/confinement.rs` and regression tests of
  the size guarantee in `tests/guarantee.rs`. The examples of `README.md` are
  run as documentation tests when all features are enabled.
- Crate-level documentation.

### Changed

- `try_retain` never takes the map below `MIN_LEN`: it stops removing at the
  lower bound, keeps the remaining rejected elements and returns `Undersize`.
- `try_from_iter` and `from_iter_checked` stop reading the iterator as soon as
  the collection exceeds `MAX_LEN`. `Error::Oversize` reports the length at
  which the iterator was abandoned, not the total number of its items.
- `HashMap::reserve` reserves space for `min(additional, MAX_LEN - len)`
  elements.
- The `ascii` feature enables `ascii/alloc`.
- The `serde` feature enables `serde/alloc`, and `std` enables `serde/std`
  when `serde` is on.

### Removed

- `src/confinement.rs`, the single-file version of the crate which was not a
  part of the module tree. The deprecated `confined_*!` macros and the
  `smallvec` aliases and macros it contained are not ported.
