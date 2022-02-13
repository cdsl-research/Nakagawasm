from __future__ import annotations
import asyncio
import numpy as np
import matplotlib.pylab as plt
import japanize_matplotlib
import csv


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
    japanize_matplotlib.japanize()

    c = np.random.normal(loc=512, scale=512, size=1024)
    c += abs(c.min())

    plt.hist(c, bins=100)
    plt.xlabel("ファイルサイズ (kBytes)", fontsize=14)
    plt.ylabel("ファイル数", fontsize=14)
    plt.savefig("hoge.png", bbox_inches='tight', pad_inches=0)
    plt.savefig("hoge.pdf", bbox_inches='tight', pad_inches=0)

    for i, n in enumerate(c):
        await dd(str(i), n)

    tasks = [dd(str(i), n) for i, n in enumerate(c)]


if __name__ == "__main__":
    asyncio.run(main())
