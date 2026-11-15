# Changelog

## [Unreleased]

### Added

- Any iterable of category names (sets, generators, dict keys) in `categories` and `exclude_categories` of `query`.

### Removed

- Free-threaded CPython 3.13 wheels.

### Performance

- Category names in `query` parsed without copying each one, ~2x less time per name.

## [0.3.1] - 2026-06-06

### Performance

- `query` with a high `min_codepoint` binary-searches past the leading category intervals below the range instead of scanning them, making bounded queries near the top of the codepoint space up to ~4.5x faster.
- `query` with `max_codepoint` below 256 and ASCII-only custom characters uses a 256-bit bitset fast path, making small-range queries several times faster.
- `categories` caches its result tuple per Unicode version, making repeated calls ~38x faster.

## [0.3.0] - 2026-06-06

- Initial release.
