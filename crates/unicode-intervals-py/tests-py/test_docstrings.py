import unicode_intervals


def test_public_symbols_have_docstrings():
    for name in ("query", "categories", "as_general_categories", "UnicodeVersion"):
        assert getattr(unicode_intervals, name).__doc__, f"{name} is missing a docstring"
    assert unicode_intervals.__doc__, "module is missing a docstring"
