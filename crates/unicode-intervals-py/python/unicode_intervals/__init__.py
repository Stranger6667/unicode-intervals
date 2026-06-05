"""Search Unicode code point intervals by category, codepoint range, and characters."""

from .unicode_intervals import (
    UnicodeVersion,
    __version__,
    as_general_categories,
    available_versions,
    categories,
    query,
)

__all__ = [
    "UnicodeVersion",
    "as_general_categories",
    "available_versions",
    "categories",
    "query",
    "__version__",
]
