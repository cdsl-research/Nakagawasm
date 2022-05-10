from __future__ import annotations

from pathlib import Path
from typing import Final

TARGETDIR: Final[Path] = Path("static")


def size(p: Path) -> tuple[str, int]:
    if not p.is_dir():
        return p.name, p.stat().st_size
    else:
        return p.name, sum(map(lambda c: c.stat().st_size, p.glob("**/*")))


def main() -> None:
    it = map(size, TARGETDIR.iterdir())
    res = sorted(it, reverse=True, key=lambda x: x[1])
    print(*enumerate(res, start=1), sep="\n")
    print(f"total: {sum(map(lambda x: x[1], res)):,}")


if __name__ == "__main__":
    main()
