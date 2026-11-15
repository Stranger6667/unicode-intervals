# Changelog

## [Unreleased]

### Added

- Wheels for Linux on armv7, ppc64le, s390x and riscv64, for musllinux on i686, armv7, ppc64le and riscv64, and for Windows on ARM64.
- PyPy 3.12 wheels, and PyPy wheels for macOS on x86-64.
- Pyodide wheel on PyPI.
- `abi3t` wheels: one per platform for CPython 3.15+, free-threaded builds included.

### Changed

- aarch64 manylinux wheels require glibc 2.17 instead of 2.28.

## [0.5.0] - 2026-11-15

### Added

- Support for Unicode 18.0.0.
- Support for Python 3.15, including free-threaded wheels.
- Hashing for `UnicodeVersion`, so it works as a dict key and set member.

### Changed

- **BREAKING**: `UnicodeVersion.latest()` and the default `version` are Unicode 18.0.0.
- Building from source requires Rust 1.89.
- `repr()` of `UnicodeVersion` is `UnicodeVersion.V16_0_0` instead of `UnicodeVersion("16.0.0")`: the custom `__repr__` made free-threaded Python 3.15 emit a `DeprecationWarning` at import.

### Performance

- `query` with several categories uses NEON on AArch64, making such queries ~2.2-2.6x faster on Apple silicon.
- `query` with several categories uses AVX-512 where available, making such queries ~10% faster than with AVX2.

## [0.4.0] - 2026-11-15

### Added

- Any iterable of category names (sets, generators, dict keys) in `categories` and `exclude_categories` of `query`.

### Removed

- Free-threaded CPython 3.13 wheels.

### Performance

- Category names in `query` parsed without copying each one, ~2x less time per name.
- `query` with several categories scans a single partition table of all categories instead of concatenating and re-sorting per-category intervals (with AVX2 on x86-64).
- `query` with `include_characters` on top of categories merges the characters into the sorted result instead of re-sorting it.
- `query` with a single category and a codepoint range copies the matching intervals in bulk.

## [0.3.1] - 2026-06-06

### Performance

- `query` with a high `min_codepoint` binary-searches past the leading category intervals below the range instead of scanning them, making bounded queries near the top of the codepoint space up to ~4.5x faster.
- `query` with `max_codepoint` below 256 and ASCII-only custom characters uses a 256-bit bitset fast path, making small-range queries several times faster.
- `categories` caches its result tuple per Unicode version, making repeated calls ~38x faster.

## [0.3.0] - 2026-06-06

- Initial release.
