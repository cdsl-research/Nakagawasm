from __future__ import annotations

import asyncio
from pathlib import Path

import japanize_matplotlib
import matplotlib.pylab as plt
import numpy as np

OUT = "files"


async def dd(filename: str, size: int):
    proc = await asyncio.create_subprocess_shell(
        f"dd if=/dev/zero of=dds/{filename} bs=1024 count={int(size)}",
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE)
    stdout, stderr = await proc.communicate()

    buf = str()

    if stdout:
        buf += stdout.decode()
    if stderr:
        buf += stderr.decode()

    return buf


async def main() -> None:
    c = np.random.normal(loc=512, scale=512, size=1024)
    c += abs(c.min())

    plt.hist(c, bins=100)
    plt.xlabel("ファイルサイズ (kBytes)", fontsize=14)
    plt.ylabel("ファイル数", fontsize=14)
    plt.savefig("hoge.png", bbox_inches='tight', pad_inches=0)
    plt.savefig("hoge.pdf", bbox_inches='tight', pad_inches=0)

    # for i, n in enumerate(c):
    #     await dd(str(i), n)

    asyncio.gather(*[dd(str(i), n) for i, n in enumerate(c)])


def graph_regenerate() -> None:
    it = map(lambda p: p.stat().st_size // 1024,
             Path("../workdir/dds/").iterdir())
    plt.hist(list(it), bins=100)
    plt.grid()
    plt.xlabel("ファイルサイズ (kBytes)", fontsize=14)
    plt.ylabel("ファイル数", fontsize=14)
    plt.savefig(f"{OUT}.png", bbox_inches='tight', pad_inches=0)
    plt.savefig(f"{OUT}.pdf", bbox_inches='tight', pad_inches=0)


if __name__ == "__main__":
    japanize_matplotlib.japanize()

    # asyncio.run(main())
    graph_regenerate()
