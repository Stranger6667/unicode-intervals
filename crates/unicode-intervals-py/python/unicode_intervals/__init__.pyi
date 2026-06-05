from collections.abc import Iterable

class UnicodeVersion:
    V9_0_0: UnicodeVersion
    V10_0_0: UnicodeVersion
    V11_0_0: UnicodeVersion
    V12_0_0: UnicodeVersion
    V12_1_0: UnicodeVersion
    V13_0_0: UnicodeVersion
    V14_0_0: UnicodeVersion
    V15_0_0: UnicodeVersion
    V15_1_0: UnicodeVersion
    V16_0_0: UnicodeVersion
    V17_0_0: UnicodeVersion
    def __new__(cls, value: str) -> UnicodeVersion: ...
    @staticmethod
    def latest() -> UnicodeVersion: ...
    def __str__(self) -> str: ...

available_versions: tuple[UnicodeVersion, ...]
__version__: str

def query(
    *,
    categories: Iterable[str] | None = ...,
    exclude_categories: Iterable[str] | None = ...,
    min_codepoint: int | None = ...,
    max_codepoint: int | None = ...,
    include_characters: str = ...,
    exclude_characters: str = ...,
    version: UnicodeVersion | None = ...,
) -> tuple[tuple[int, int], ...]: ...
def categories(version: UnicodeVersion | None = ...) -> tuple[str, ...]: ...
def as_general_categories(
    categories: Iterable[str], version: UnicodeVersion | None = ...
) -> tuple[str, ...]: ...
