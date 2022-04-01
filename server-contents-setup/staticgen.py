from __future__ import annotations

import re
import subprocess
from pathlib import Path
from sys import stderr
from typing import Final, Iterator
from urllib.parse import ParseResult, urlparse

import requests

OUTDIR: Final[Path] = Path("static")


def setup_saved_dir(url: ParseResult) -> Path:
    path = OUTDIR / Path(url.path).name
    path.mkdir(exist_ok=True)
    return path


def img_links(html: str) -> Iterator[ParseResult]:
    """ """
    result = subprocess.run(
        "htmlq -a src img".split(),
        encoding="utf-8",
        input=html,
        text=True,
        check=True,
        stdout=subprocess.PIPE,
    )
    return map(urlparse, result.stdout.splitlines())


def clean_html(html: str) -> str:
    return subprocess.run(
        "htmlq -wt --remove-nodes script --remove-nodes style".split(),
        encoding="utf-8",
        input=html,
        text=True,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout


def replace_img(html: str, imgname: str) -> str:
    pat = f"<img.*{imgname}.*>"
    ret = re.sub(pat, f'<img src="{imgname}">', html)
    return ret


def save(url: ParseResult, d: Path) -> None:
    # get html
    r = requests.get(url.geturl())
    r.raise_for_status()

    html = r.content.decode(encoding="utf-8")
    html = clean_html(html)

    # get images
    for img in list(img_links(html)):
        r = requests.get(img.geturl())
        try:
            r.raise_for_status()
        except requests.HTTPError as e:
            print(e, file=stderr)

        img_fname = Path(img.path).name

        # update IMGPATH: `<img src="[IMGPATH]" ... >`
        html = replace_img(html, img_fname)
        # save image file
        with open(d / img_fname, mode="wb") as f:
            f.write(r.content)

    # save html
    with open(d / "index.html", mode="w", encoding="utf-8") as f:
        f.write("""<head><meta charset="utf-8"></head>""" + html)


def main() -> None:
    OUTDIR.mkdir(exist_ok=True)

    with open("all-post-links.txt") as f:
        for url in map(urlparse, f):
            d = setup_saved_dir(url)
            if not (d / "index.html").exists():
                try:
                    save(url, d)
                except requests.HTTPError as e:
                    print(e, file=stderr)
            else:
                print(f"skip {d.name}")


def test() -> None:
    with open("hoge.html") as f:
        html = f.read()
    # html = clean_html(html)
    for img in list(img_links(html)):
        html = replace_img(html, Path(img.geturl()).name)
    print(html)


if __name__ == "__main__":
    main()
    # test()
