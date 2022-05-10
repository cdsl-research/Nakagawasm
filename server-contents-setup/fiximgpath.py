from __future__ import annotations

import re
from pathlib import Path
from typing import Generator


def lookup_all_html(path: Path) -> Generator[Path, None, None]:
    return path.glob("**/*.html")


if __name__ == "__main__":
    pattern = re.compile(r"""^<img src="(.*)">$""")
    for html_path in lookup_all_html(Path("static")):
        html = html_path.read_text()
