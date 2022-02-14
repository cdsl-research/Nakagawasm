from __future__ import annotations

from pathlib import Path

import japanize_matplotlib
import pandas as pd
from matplotlib import pyplot as plt

# TH = "6859"
TH = "none"


def main() -> None:
    parent_dir = Path(f"../workdir/log/{TH}")
    print(parent_dir)
    files: map[Path] = map(lambda p: p / "mem.csv", parent_dir.glob("*"))

    plt.figure()

    for file in files:
        df = pd.read_csv(
            file,
            header=None,
            #  index_col="datetime",
            names=["datetime", "uss"],
            parse_dates=[0])
        plt.plot(range(0, 590, 10), df["uss"].values)

    plt.grid()
    plt.xlim(0, 600)
    plt.ylim(0, 8_500)
    plt.tick_params("both", labelsize=12)
    plt.xlabel("経過時間 (sec)", fontsize=14)
    plt.ylabel("メモリ使用量 (kBytes)", fontsize=14)
    plt.savefig(f"{TH}-spray.png", bbox_inches='tight', pad_inches=0)
    plt.savefig(f"{TH}-spray.pdf", bbox_inches='tight', pad_inches=0)
    plt.close("all")


if __name__ == "__main__":
    japanize_matplotlib.japanize()
    main()
