import pytest

import unicode_intervals

VERSION = unicode_intervals.UnicodeVersion("15.0.0")

QUERY_CASES = {
    "all": {},
    "ascii range": {"max_codepoint": 128},
    "one category - ascii": {"categories": ["Lu"], "max_codepoint": 128},
    # Many names, few intervals: dominated by argument parsing.
    "many category names - ascii": {
        "categories": ["Lu", "Ll", "Lt", "Lm", "Lo", "Nd", "Nl", "No"],
        "exclude_categories": ["Cc", "Cs", "Zs", "Zl", "Zp"],
        "max_codepoint": 128,
    },
    "include and exclude characters - ascii": {
        "exclude_categories": ["Lu"],
        "include_characters": "0123456789",
        "exclude_characters": "QWERTYUIOP",
        "max_codepoint": 128,
    },
    "exclude characters beyond ascii": {
        "exclude_categories": ["Lu"],
        "exclude_characters": "A@т",
        "max_codepoint": 128,
    },
    "exclude one category - full range": {"exclude_categories": ["Cs"]},
    # Hundreds of intervals: dominated by building the result tuple.
    "one category - full range": {"categories": ["Lo"]},
    "many categories - full range": {"categories": ["Lu", "Ll", "Lo", "Nd", "Po", "Sm"]},
    "one category - codepoint range": {"categories": ["Lo"], "max_codepoint": 0x10000},
}

AS_GENERAL_CATEGORIES_CASES = {
    "one major class": ["N"],
    "all major classes": ["L", "M", "N", "P", "S", "Z", "C"],
    "explicit": ["Lu", "Ll", "Nd"],
}


@pytest.mark.parametrize("case", list(QUERY_CASES))
def test_query(benchmark, case):
    kwargs = {**QUERY_CASES[case], "version": VERSION}
    benchmark(lambda: unicode_intervals.query(**kwargs))


def test_categories(benchmark):
    benchmark(unicode_intervals.categories, VERSION)


@pytest.mark.parametrize("case", list(AS_GENERAL_CATEGORIES_CASES))
def test_as_general_categories(benchmark, case):
    benchmark(unicode_intervals.as_general_categories, AS_GENERAL_CATEGORIES_CASES[case], VERSION)
