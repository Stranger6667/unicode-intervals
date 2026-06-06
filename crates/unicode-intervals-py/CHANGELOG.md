# Changelog

## [Unreleased]

### Performance

- `query` with a high `min_codepoint` binary-searches past the leading category intervals below the range instead of scanning them, making bounded queries near the top of the codepoint space up to ~4.5x faster.
- `query` with `max_codepoint` below 256 and ASCII-only custom characters uses a 256-bit bitset fast path, making small-range queries up to ~5x faster.

## [0.3.0] - 2026-06-06

- Initial release.
