import sys
import unicodedata

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st
from hypothesis.internal import charmap

import unicode_intervals

_BUNDLED = {str(v) for v in unicode_intervals.available_versions}
pytestmark = pytest.mark.skipif(
    unicodedata.unidata_version not in _BUNDLED,
    reason=f"running Python Unicode {unicodedata.unidata_version} is not bundled",
)

VERSION = unicode_intervals.UnicodeVersion(unicodedata.unidata_version)

# charmap category order is cache-dependent (scan order cold, abbreviation once cached).
# Force the cached/steady-state order our binding targets before comparing.
charmap.charmap()
charmap._charmap = None
charmap._categories = None
charmap.charmap()

ALL_CATEGORIES = list(charmap.categories())

category_lists = st.lists(st.sampled_from(ALL_CATEGORIES), unique=True) | st.none()
codepoints = st.integers(min_value=0, max_value=sys.maxunicode)


@settings(max_examples=300)
@given(
    categories=category_lists,
    lo=codepoints,
    hi=codepoints,
    include=st.text(max_size=5),
    exclude=st.text(max_size=5),
)
def test_query_matches_charmap(categories, lo, hi, include, exclude):
    lo, hi = sorted((lo, hi))
    expected = charmap.query(
        categories=categories,
        min_codepoint=lo,
        max_codepoint=hi,
        include_characters=include,
        exclude_characters=exclude,
    ).intervals
    got = unicode_intervals.query(
        categories=categories,
        min_codepoint=lo,
        max_codepoint=hi,
        include_characters=include,
        exclude_characters=exclude,
        version=VERSION,
    )
    assert got == expected


def test_categories_matches_charmap():
    assert unicode_intervals.categories(VERSION) == charmap.categories()


@given(st.lists(st.sampled_from(ALL_CATEGORIES + ["L", "M", "N", "P", "S", "Z", "C"]), unique=True))
def test_as_general_categories_matches_charmap(categories):
    assert unicode_intervals.as_general_categories(categories, VERSION) == charmap.as_general_categories(categories)
