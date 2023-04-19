import sys
import tempfile
import urllib
import subprocess
import urllib.request
import zipfile
import pathlib

CATEGORIES = {
    'Close_Punctuation',
    'Connector_Punctuation',
    'Control',
    'Currency_Symbol',
    'Dash_Punctuation',
    'Decimal_Number',
    'Enclosing_Mark',
    'Final_Punctuation',
    'Format',
    'Initial_Punctuation',
    'Letter_Number',
    'Line_Separator',
    'Lowercase_Letter',
    'Math_Symbol',
    'Modifier_Letter',
    'Modifier_Symbol',
    'Nonspacing_Mark',
    'Open_Punctuation',
    'Other_Letter',
    'Other_Number',
    'Other_Punctuation',
    'Other_Symbol',
    'Paragraph_Separator',
    'Private_Use',
    'Space_Separator',
    'Spacing_Mark',
    'Surrogate',
    'Titlecase_Letter',
    'Unassigned',
    'Uppercase_Letter',
}


def fetch(unicode_version: str, directory: str) -> None:
    response, _ = urllib.request.urlretrieve(f"https://www.unicode.org/Public/zipped/{unicode_version}/UCD.zip")
    with zipfile.ZipFile(response, "r") as compressed:
        compressed.extractall(directory)


def transform_code(code: bytes) -> bytes:
    code = code.replace(
        b"pub const BY_NAME: &'static [(&'static str, &'static [(u32, u32)])]",
        b"pub const BY_NAME: &'static [&'static [(u32, u32)]]"
    )
    lines = code.splitlines()
    for category in CATEGORIES:
        idx = next((i for i, line in enumerate(lines) if category.encode() in line), None)
        lines[idx] = lines[idx].replace(f"(\"{category}\", ".encode(), b"").replace(b"),", b",")
    return b"\n".join(lines)


def ucd_generate(directory: str) -> bytes:
    result = subprocess.run(
        [
            f"ucd-generate general-category {directory} "
            # Exclude groups that include other groups
            f"--exclude=Separator,Symbol,Cased_Letter,Letter,Mark,Number,Other,Punctuation",
        ],
        capture_output=True,
        shell=True,
        check=True,
    )
    return result.stdout


def get_output_path(unicode_version: str) -> pathlib.Path:
    tables_directory = pathlib.Path("src/tables")
    tables_directory.mkdir(parents=True, exist_ok=True)
    version = unicode_version.replace(".", "_")
    return tables_directory / f"v{version}.rs"


USAGE = """Usage: python generate_tables.py <unicode version>"""


def main() -> None:
    if len(sys.argv) != 2:
        print(USAGE)
        sys.exit(1)
    unicode_version = sys.argv[1]
    temp_directory = tempfile.mkdtemp()
    fetch(unicode_version, temp_directory)
    code = ucd_generate(temp_directory)
    code = transform_code(code)
    with get_output_path(unicode_version).open("wb") as fd:
        fd.write(code)


if __name__ == "__main__":
    main()
