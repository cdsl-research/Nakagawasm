from __future__ import annotations

import asyncio
from pathlib import Path
from typing import Optional

import japanize_matplotlib
import matplotlib.pylab as plt
import numpy as np
from numpy.typing import NDArray

OUT = "files"


async def dd(filename: str, size: int) -> str:
    proc = await asyncio.create_subprocess_shell(
        f"dd if=/dev/zero of=../workdir/dds/{filename} bs=1024 count={size}",
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE)
    stdout, stderr = await proc.communicate()

    buf = str()

    if stdout:
        buf += stdout.decode()
    if stderr:
        buf += stderr.decode()

    return buf


async def generate_files() -> NDArray[np.int64]:
    c: NDArray[np.int64] = np.random.normal(loc=512, scale=512,
                                            size=1024).astype(np.int64)
    c += abs(c.min())
    result = await asyncio.gather(
        *[dd(str(i), int(n)) for i, n in enumerate(c)])
    print(result)
    return c


def make_graph(c: Optional[NDArray[np.int64]] = None) -> None:
    if c is None:
        it = map(lambda p: p.stat().st_size // 1024,
                 Path("../workdir/dds/").iterdir())
        c = np.array(list(it), dtype=np.int64)
    hist, bin_edges = np.histogram(c, bins=32)
    bin_edges = (bin_edges[:-1] + bin_edges[1:]) // 2

    fig, ax = plt.subplots()

    ax.grid()
    ax.set_xlim(0, 3250)
    ax.set_ylim(0, 90)
    ax.set_xlabel("ファイルサイズ　[KiB]", fontsize=14)
    ax.set_ylabel("ファイル数", fontsize=14)

    ax.plot(bin_edges, hist, ".-")

    fig.savefig(f"out/{OUT}.png", bbox_inches='tight', pad_inches=0)
    fig.savefig(f"out/{OUT}.pdf", bbox_inches='tight', pad_inches=0)


async def main() -> None:
    c = await generate_files()
    make_graph(c)


if __name__ == "__main__":
    japanize_matplotlib.japanize()
    if False:
        asyncio.run(main())
    else:
        make_graph()
