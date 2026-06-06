import unicodedata

import pytest
from hypothesis.internal import charmap

import unicode_intervals

charmap.charmap()  # warm the one-time on-disk table build

_BUNDLED = {str(v) for v in unicode_intervals.available_versions}
VERSION = (
    unicode_intervals.UnicodeVersion(unicodedata.unidata_version)
    if unicodedata.unidata_version in _BUNDLED
    else unicode_intervals.UnicodeVersion.latest()
)

ROUNDS = 50

QUERY_CASES = {
    "all": {},
    "ascii_range": {"min_codepoint": 0, "max_codepoint": 128},
    "single_category": {"categories": ["Lu"]},
    "single_category_range": {
        "categories": ["Lu"],
        "min_codepoint": 0,
        "max_codepoint": 0x2FFF,
    },
    "many_categories": {"categories": ["Lu", "Ll", "Nd", "Po", "Sm", "Sk", "Cc"]},
    "include_exclude": {
        "categories": ["Lu"],
        "include_characters": "☃-∞",
        "exclude_characters": "AOZ",
    },
}

AS_GENERAL_CATEGORIES_CASES = {
    "expand_major": ["N"],
    "all_majors": ["L", "M", "N", "P", "S", "Z", "C"],
    "explicit": ["Lu", "Ll", "Nd"],
}


def _clear_query_caches():
    charmap.category_index_cache.clear()
    charmap.category_index_cache[
        frozenset()
    ] = ()  # empty-key base case, seeded at import
    charmap.limited_category_index_cache.clear()


@pytest.mark.parametrize("implementation", ["hypothesis", "unicode_intervals"])
@pytest.mark.parametrize("case", list(QUERY_CASES))
def test_query(benchmark, implementation, case):
    benchmark.group = f"query: {case}"
    query_kwargs = QUERY_CASES[case]
    if implementation == "hypothesis":

        def setup():
            _clear_query_caches()
            return (), query_kwargs

        benchmark.pedantic(charmap.query, setup=setup, rounds=ROUNDS)
    else:
        our_kwargs = {**query_kwargs, "version": VERSION}
        benchmark.pedantic(unicode_intervals.query, kwargs=our_kwargs, rounds=ROUNDS)


@pytest.mark.parametrize("implementation", ["hypothesis", "unicode_intervals"])
def test_categories(benchmark, implementation):
    benchmark.group = "categories"
    if implementation == "hypothesis":

        def setup():
            charmap._categories = None
            return (), {}

        benchmark.pedantic(charmap.categories, setup=setup, rounds=ROUNDS)
    else:
        benchmark.pedantic(unicode_intervals.categories, args=(VERSION,), rounds=ROUNDS)


@pytest.mark.parametrize("implementation", ["hypothesis", "unicode_intervals"])
@pytest.mark.parametrize("case", list(AS_GENERAL_CATEGORIES_CASES))
def test_as_general_categories(benchmark, implementation, case):
    benchmark.group = f"as_general_categories: {case}"
    categories = AS_GENERAL_CATEGORIES_CASES[case]
    if implementation == "hypothesis":
        benchmark.pedantic(charmap.as_general_categories, args=(categories,), rounds=ROUNDS)
    else:
        benchmark.pedantic(
            unicode_intervals.as_general_categories, args=(categories, VERSION), rounds=ROUNDS
        )
