# Changelog

## [Unreleased]

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

[Unreleased]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.2.0...HEAD
[0.2.0]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.1.2...rust-v0.2.0
[0.1.2]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.1.1...rust-v0.1.2
[0.1.1]: https://github.com/Stranger6667/unicode-intervals/compare/rust-v0.1.0...rust-v0.1.1
