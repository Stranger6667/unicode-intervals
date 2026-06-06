# Changelog

## [Unreleased]

### Performance

- `query` with a high `min_codepoint` binary-searches past the leading category intervals below the range instead of scanning them, making bounded queries near the top of the codepoint space up to ~4.5x faster.
- `query` with `max_codepoint` below 256 and ASCII-only custom characters uses a 256-bit bitset fast path, making small-range queries up to ~5x faster.

## [0.3.0] - 2026-06-06

### Added

- Support for Unicode 15.1.0, 16.0.0, and 17.0.0.
- `IntoIterator` for `&IntervalSet`, exposed via the new `Codepoints` iterator type.
- `as_general_categories` to expand major category classes (e.g. `N` -> `Nd, Nl, No`) and validate category names.
- `UnicodeVersion::ALL` listing every bundled Unicode version.

### Changed

- **BREAKING**: `UnicodeVersion::latest()` now returns `UnicodeVersion::V17_0_0`.
- **BREAKING**: `UnicodeVersion` is now `#[non_exhaustive]`, so future Unicode versions can be added without a breaking change.
- **BREAKING**: Bump the Minimum Supported Rust Version to 1.85.

### Removed

- **BREAKING**: `UnicodeVersion::intervals` and `UnicodeVersion::interval_set`. Use the `query()` builder instead.

### Performance

- `IntervalSet::index_of`, `index_above`, and `contains` now use binary search (`O(log n)`) instead of a linear scan.
- Skip a redundant sort/merge in `query` for single-category, all-category, and empty queries, whose intervals are already sorted.
- `query` with a restricted codepoint range no longer materializes and scans the entire category set; it collects only intervals within the range, stopping early per category.

## [0.2.0] - 2023-04-25

- Reduce the size of the struct returned by `UnicodeCategorySet.iter()` from 8 to 4 bytes and improve performance for
  cases when a few Unicode categories are involved.
- Reduce allocations inside `query` for cases when a few Unicode categories are involved.
- Rename `UnicodeCategorySet::add_category` to `UnicodeCategorySet::add` and `UnicodeCategorySet::has_category` to `UnicodeCategorySet::contains`.
- Add `UnicodeCategorySet::remove`.
- Only sort intervals after applying the codepoints filter.

## [0.1.2] - 2023-04-23

- Support `DoubleEndedIterator` for `IntervalSet.iter()`.

## [0.1.1] - 2023-04-23

- Documentation updates.

## 0.1.0 - 2023-04-23

- Initial public release.

[Unreleased]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.3.0...HEAD
[0.3.0]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.2.0...rust-v0.3.0
[0.2.0]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.1.2...rust-v0.2.0
[0.1.2]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.1.1...rust-v0.1.2
[0.1.1]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.1.0...rust-v0.1.1
