import pytest

import unicode_intervals


def test_versions():
    assert str(unicode_intervals.UnicodeVersion.latest()) == "17.0.0"
    assert str(unicode_intervals.UnicodeVersion("16.0.0")) == "16.0.0"
    assert unicode_intervals.UnicodeVersion("16.0.0") == unicode_intervals.UnicodeVersion.V16_0_0
    assert unicode_intervals.UnicodeVersion.latest() in unicode_intervals.available_versions
    assert len(unicode_intervals.available_versions) == 11


def test_unknown_version():
    with pytest.raises(ValueError):
        unicode_intervals.UnicodeVersion("99.0.0")


def test_available_versions_roundtrip():
    versions = unicode_intervals.available_versions
    labels = [str(v) for v in versions]
    assert len(set(labels)) == len(versions)  # no duplicate mappings
    for version in versions:
        assert unicode_intervals.UnicodeVersion(str(version)) == version


def test_query_examples():
    assert unicode_intervals.query() == ((0, 1114111),)
    assert unicode_intervals.query(min_codepoint=0, max_codepoint=128) == ((0, 128),)
    assert unicode_intervals.query(min_codepoint=0, max_codepoint=128, categories=["Lu"]) == ((65, 90),)
    assert unicode_intervals.query(
        min_codepoint=0, max_codepoint=128, categories=["Lu"], include_characters="☃"
    ) == ((65, 90), (9731, 9731))


def test_query_version_and_excludes():
    res = unicode_intervals.query(
        categories=["Lu"],
        min_codepoint=0,
        max_codepoint=128,
        exclude_characters="AB",
        version=unicode_intervals.UnicodeVersion("15.0.0"),
    )
    assert res == ((67, 90),)


def test_query_invalid():
    with pytest.raises(ValueError):
        unicode_intervals.query(categories=["Xx"])
    with pytest.raises(ValueError):
        unicode_intervals.query(min_codepoint=10, max_codepoint=5)


def test_categories():
    categories = unicode_intervals.categories()
    assert isinstance(categories, tuple)
    assert categories[-2:] == ("Cc", "Cs")
    assert {"Lu", "Ll", "Nd"} <= set(categories)


def test_as_general_categories():
    assert isinstance(unicode_intervals.as_general_categories(["N"]), tuple)
    assert set(unicode_intervals.as_general_categories(["N"])) == {"Nd", "Nl", "No"}
    with pytest.raises(ValueError):
        unicode_intervals.as_general_categories(["Xx"])
