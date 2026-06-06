# Changelog

## [Unreleased]

### Performance

- `query` with a high `min_codepoint` binary-searches past the leading category intervals below the range instead of scanning them, making bounded queries near the top of the codepoint space up to ~4.5x faster.

## [0.3.0] - 2026-06-06

- Initial release.
