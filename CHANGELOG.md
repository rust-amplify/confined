Change Log
==========

0.5.0 (unreleased)
------------------

This version closes the paths by which a confined collection could get outside
of the limits of its type, and restores the type aliases, macros and tests
which were left behind when the crate was split out of `amplify`. It also
migrates to the current `collections` traits, adds allocator-aware vectors,
linked lists and heaps, and requires Rust 1.100.0. Some changes affect the
public API.

### Breaking changes

| Change | Migration |
|--------|-----------|
| The MSRV is Rust 1.100.0 and the crate uses edition 2024. | Update the Rust toolchain; dependent crates need not change their edition. |
| `VecDeque`: `pop_front`, `pop_back`, `truncate` and `drain` exist only when `MIN_LEN` is 0, like `pop` of `Vec`. | Use `remove(index)`, which checks the lower bound. |
| `entry()` returns `confined::Entry` instead of the entry of the wrapped map. The new type can't remove the entry. | Use `key`, `or_insert`, `or_insert_with`, `or_default` and `and_modify`; `or_insert_with_key` is available only for built-in maps. Remove with `remove(&key)` of the confined map. |
| The blanket `FromStr` implementation is replaced with implementations for `String` and `AsciiString`. `FromStr::Err` is `Error` for `String` (it was `Infallible`) and `AsciiError` for `AsciiString` (it was `AsAsciiStrError`). | Handle the error. |
| `"...".parse()`, `FromHex` and `Deserialize` reject the data which do not fit the limits. `FromHex` reports `hex::Error::InvalidLength` with the violated limit and the number of bytes. | None. The data which were accepted in violation of the limits are an error now. |
| `Confined` has no struct-level `Collection` bound; its size-aware generic methods require `C: ItemLen`. `ItemLen` now has `Item` and `Access` associated types. | Add `ItemLen` to generic bounds. For your own `Collection`, use `impl ItemLen for MyCollection { type Item = <Self as collections::Collection>::Item; type Access = confined::ViaCollection; }`. For `CollectionAlloc`, use its `Item` and `confined::ViaCollectionAlloc` instead. |
| The map API uses `CollectionMap` and `MapEntry` instead of `KeyedCollection`; set operations use `CollectionSet`. | Update custom collection implementations to the new `collections` traits. Custom map entries implementing `MapEntry` get the generic `Entry` methods. |
| `with()`, `with_in()` and `with_key_value()` fail to build unless `MIN_LEN <= 1 <= MAX_LEN`. `with()` and `with_in()` also require `MAX_LEN >= MAX_ITEM_LEN` (`char::MAX_LEN_UTF8` for `String`). | Construct a value with a valid length using `try_from` or `parse`. |
| `try_from` and the constructors built on it fail to build for a type with `MIN_LEN > MAX_LEN`. | Fix the limits. Such a type could never hold a value. |
| `BTreeMap::append` takes a source map with `MIN_LEN = 0` and has one const parameter instead of two. | Release a source which has a lower bound, and confine it again with `MIN_LEN = 0`. |
| The deprecated methods are removed. | `from_collection_unsafe` → `from_checked`, `from_iter_unsafe` → `from_iter_checked`, `from_slice_unsafe` → `from_slice_checked`, `as_inner` → `as_unconfined`, `to_inner` → `to_unconfined`, `into_inner` and `unbox` → `release`, `push_from` → `push_front`. |
| The `indexmap` feature no longer enables `std`; default-hasher index aliases and macros require both features. | Enable `std` explicitly when using those aliases or macros with default features disabled. For `no_std`, wrap `IndexMap<K, V, S>` or `IndexSet<T, S>` directly in `Confined`, with `S: BuildHasher + Default`. |
| `first()` and `last()` on indexable collections, and `first_mut()` and `last_mut()` where mutable indexing is supported, now resolve to the confined accessors for all `MIN_LEN` values; calling them requires `MIN_LEN > 0` at build time. | For an optional result, use `as_slice().first()` / `as_slice().last()` or call the wrapped collection through `as_unconfined()`. Mutable optional accessors are available through `as_mut()` when the collection implements `AsMut`. |

The inline const bound failures listed above are reported by `cargo build`
and `cargo test`, but not by `cargo check`.

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
  `hex`, `serde`, `ascii` or `indexmap` individually.
- Unresolved documentation links to `u24::MAX` and `mem::forget`.

### Added

- Type aliases for the common limits, exported from the crate root: `Tiny*`,
  `Small*`, `Medium*`, `Large*`, `Confined*` and `NonEmpty*` for `String`,
  `AsciiString` (`*Ascii`), `Vec<u8>` (`*Blob`), `Vec`, `VecDeque` (`*Deque`),
  `LinkedList` (`*List`), `BinaryHeap` (`*Heap`), `HashSet`, `BTreeSet`
  (`*OrdSet`), `HashMap`, `BTreeMap` (`*OrdMap`), `IndexSet` and `IndexMap`.
  `ConfinedHashMap` wraps `HashMap<K, V>`; the unported version wrapped
  `HashSet<K, V>`.
- Constructor macros `tiny_*!`, `small_*!` and `medium_*!` for `s`, `blob`,
  `vec`, `deque`, `list`, `heap`, `set`, `bset`, `map`, `bmap`, `iset` and
  `imap`. They do not need any other macro or import in the scope of the
  caller.
- `LinkedList` and `BinaryHeap` confinement. Lists support `push_front`,
  `push_back` and checked `remove(index)`; `pop_front` / `pop_back` on lists
  and `pop` on heaps require `MIN_LEN = 0`.
- Allocator-aware `Vec<T, A: Allocator>` support through the same `Confined`
  type. `new_in`, `with_capacity_in`, `with_in`, `try_from_iter_in` and
  `from_iter_checked_in` accept an allocator explicitly without requiring
  `A: Default`, including for single-item construction with `with_in`.
- An allocator parameter defaulting to `Global` on all `*Vec` and `*Blob`
  aliases, for example `TinyVec<T, A = Global>` and `TinyBlob<A = Global>`.
- `Entry`, the type returned by `entry()` of a confined map, with the methods
  `key`, `or_insert`, `or_insert_with`, `or_default` and `and_modify` for any
  `CollectionMap`, including custom maps using `MapEntry`. The additional
  `or_insert_with_key` method remains specific to `BTreeMap`, `HashMap` and
  `IndexMap`.
- `ItemLen` trait, which tells how much a single item adds to the length of a
  collection: one for all supported collections except `String`, where it is
  the number of bytes in the character. Its `Item` and `Access` associated
  types select `ViaCollection` for `Collection` or `ViaCollectionAlloc` for
  `CollectionAlloc`.
- `FromStr` for confined `String` and `AsciiString`.
- `AsciiError` is exported from the crate root under the `ascii` feature.
- `first_mut()` and `last_mut()` for the collections which contain at least
  one item and can be indexed mutably by position.
- `from_array()` for `Vec`. The length of the array is checked against the
  limits when the program is built.
- `widen()`, which converts a confinement into a confinement with wider
  limits. The limits are compared when the program is built.
- Tests of the public API in `tests/confinement.rs` and regression tests of
  the size guarantee in `tests/guarantee.rs`, plus allocator coverage in
  `tests/allocator.rs` and collection integration coverage in
  `tests/collections.rs`. The examples of `README.md`, including allocator,
  list and heap usage, are run as documentation tests when all features are
  enabled.
- Crate-level documentation.

### Changed

- `collections` follows master, currently resolved in `Cargo.lock` to
  `c44b42ce383ebd9656f64f0808775debf9bda1fa`. `indexmap` is updated to 2.14.2
  and its feature no longer implies `std`.
- `HashMap`, `HashSet`, `IndexMap` and `IndexSet` implementations accept
  `S: BuildHasher + Default`; their aliases keep the default hasher.
  `IndexMap` and `IndexSet` with explicit custom hashers work in `no_std`.
- `BTreeMap` keys require `Ord` but no longer require `Hash`.
- Compile-time bound checks use inline const assertions instead of helper
  structs. String item-size bounds use `char::MAX_LEN_UTF8`, and boolean
  confinement checks use `bool::ok_or`.
- `Error` and, with `ascii`, `AsciiError` implement `core::error::Error`
  even without `std`.
- `with()` and `with_key_value()` are available for any bounds that can hold
  exactly one item. `with()` also checks the maximum item size for the
  collection.
- `first()`, `last()`, `first_mut()` and `last_mut()` are available for any
  positive `MIN_LEN`, rather than only `MIN_LEN = 1`.
- `len_u8()`, `len_u16()`, `len_u24()` and `len_u32()` are available whenever
  `MAX_LEN` does not exceed the maximum representable by their return type.
- `std::io::Write` for confined `Vec<u8, A>` is available for every `MIN_LEN`
  and any `A: Allocator`; writes append and preserve the lower bound. Hex
  formatting also supports any allocator; `FromHex` requires `A: Default`.
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

### Known limitations

- Direct `hashbrown` support remains outside the agreed migration scope
  because the upstream `collections` feature does not build.
- Set `insert` and `retain` methods remain excluded because they overlap
  with the generic map inherent methods. Set `push`, `remove` and `take`
  remain supported.
- `ascii` 1.1.0 combined with `serde` still requires `std`.
