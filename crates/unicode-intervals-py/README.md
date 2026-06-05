# unicode-intervals

Search Unicode code point intervals by category, codepoint range, and characters.
Fast Rust core (via PyO3), output-compatible with Hypothesis's `charmap`.

## Installation

```
uv pip install unicode_intervals
```

## Usage

`query` returns a tuple of inclusive `(start, end)` code point intervals:

```python
import unicode_intervals

# Everything
assert unicode_intervals.query() == ((0, 1114111),)

# Restrict to a codepoint range
assert unicode_intervals.query(min_codepoint=0, max_codepoint=127) == ((0, 127),)

# Filter by category (uppercase letters under U+0080)
assert unicode_intervals.query(categories=["Lu"], max_codepoint=128) == ((65, 90),)

# Add specific characters, even outside the range
assert unicode_intervals.query(
    categories=["Lu"], max_codepoint=128, include_characters="0123"
) == ((48, 51), (65, 90))

# Remove specific characters
assert unicode_intervals.query(
    categories=["Lu"], min_codepoint=65, max_codepoint=90, exclude_characters="AEIOU"
) == ((66, 68), (70, 72), (74, 78), (80, 84), (86, 90))

# Exclude whole categories (an extra over charmap): drop digits from U+0030..U+003A
assert unicode_intervals.query(
    exclude_categories=["Nd"], min_codepoint=48, max_codepoint=58
) == ((58, 58),)
```

## Categories

```python
import unicode_intervals

# Normalised order, control/surrogate last
assert unicode_intervals.categories()[:6] == ("Zl", "Zp", "Co", "Me", "Pc", "Zs")
assert unicode_intervals.categories()[-2:] == ("Cc", "Cs")

# Major class expanded to its subclasses
assert unicode_intervals.as_general_categories(["N"]) == ("Nl", "Nd", "No")
```

## Unicode versions

Unicode 9.0.0 through 17.0.0 are bundled; queries default to the latest.

```python
import unicode_intervals

assert len(unicode_intervals.available_versions) == 11
assert str(unicode_intervals.available_versions[-1]) == "17.0.0"

version = unicode_intervals.UnicodeVersion("11.0.0")
assert unicode_intervals.query(categories=["Lu"], max_codepoint=128, version=version) == ((65, 90),)
```

## Use with Hypothesis

The output matches `hypothesis.internal.charmap`, so it drops into the same
interval-set machinery. Pass the interpreter's Unicode version to stay aligned:

```python
import unicodedata
import unicode_intervals

version = unicode_intervals.UnicodeVersion(unicodedata.unidata_version)
intervals = unicode_intervals.query(categories=["Lu", "Ll"], version=version)
assert intervals[:2] == ((65, 90), (97, 122))
```
