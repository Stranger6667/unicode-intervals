import re
from pathlib import Path

README = Path(__file__).resolve().parent.parent / "README.md"


def test_readme_examples():
    blocks = re.findall(r"```python\n(.*?)```", README.read_text(), re.DOTALL)
    assert blocks, "no python code blocks found in README"
    for index, block in enumerate(blocks, 1):
        try:
            exec(block, {})
        except Exception as exc:
            raise AssertionError(f"README block {index} failed: {exc}\n\n{block}") from exc
