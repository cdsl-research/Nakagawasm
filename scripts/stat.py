from __future__ import annotations

from pathlib import Path

import numpy as np
import pandas as pd
from matplotlib import pyplot as plt
import japanize_matplotlib

THS: tuple[str, ...] = ("none", "6859")


def restart_count(path: Path) -> int:
    df = pd.read_csv(path,
                     header=None,
                     index_col="datetime",
                     names=["datetime", "uss"],
                     parse_dates=[0])
    val = df["uss"].values
    return np.count_nonzero((val[1:] - val[:-1]) < 0)


def mean_restart_counts() -> None:
    for th in THS:
        files = list(
            map(lambda p: p / "mem.csv",
                Path(f"../workdir/log/{th}").iterdir()))
        stats = map(restart_count, files)
        print("平均再起動回数", np.array(list(stats)).mean())


def stats() -> None:
    fig, ax = plt.subplots()
    ax.grid(True)
    ax.set_xlim(0, 590)
    ax.set_ylim(0, 7500)
    ax.tick_params("both", labelsize=12)
    ax.set_xlabel("経過時間 [秒]", fontsize=14)
    ax.set_ylabel("メモリ使用量 [KiB]", fontsize=14)

    a = []

    for th in THS:
        files = map(lambda p: p / "mem.csv",
                    Path(f"../workdir/log/{th}").iterdir())
        dfs = np.stack([
            pd.read_csv(f,
                        header=None,
                        index_col="datetime",
                        names=["datetime", "uss"],
                        parse_dates=[0])["uss"].values for f in files
        ])
        if th == "none":
            label_name = "再起動なし"
        else:
            label_name = f"閾値{th}[KiB]で再起動"
        ax.plot(np.arange(0, 590, 10),
                dfs.mean(axis=0),
                ".-",
                label=label_name)
        a.append(dfs.mean(axis=0))
    ax.legend(fontsize=12)
    fig.savefig("out/means-spray.pdf", bbox_inches='tight', pad_inches=0)
    fig.savefig("out/means-spray.png", bbox_inches='tight', pad_inches=0)

    n, t = a
    print("平均削減率", 1 - (t / n).mean())


if __name__ == "__main__":
    japanize_matplotlib.japanize()
    # mean_restart_counts()
    stats()
