from __future__ import annotations

from pathlib import Path
from typing import cast

import japanize_matplotlib
import numpy as np
import pandas as pd
from scipy import stats
from matplotlib import pyplot as plt
from numpy.typing import NDArray

japanize_matplotlib.japanize()

TH = "6859"
TH = "none"


def solve(samples: list[int]) -> float:
    # 標本分散
    S2 = np.var(samples, ddof=1)
    # 標本サイズ
    N = len(samples)
    # 標本平均
    sample_mean = np.mean(samples)

    # 自由度N-1のt分布の上側2.5%点
    t_a = stats.t.ppf(0.975, N - 1)

    # 信頼区間
    x_0 = sample_mean - (t_a * (np.sqrt(S2 / N)))
    x_1 = sample_mean + (t_a * (np.sqrt(S2 / N)))
    print(x_0, x_1)

    return float(x_1)


def mean_by_one_instance(path: Path) -> float:
    df = pd.read_csv(path,
                     header=None,
                     index_col="datetime",
                     names=["datetime", "uss"],
                     parse_dates=[0])
    # print(df.dtypes, df)
    return float(df["uss"].median())


def plot_hist(samples: list[float]) -> None:
    hist, bin_edges = cast(tuple[NDArray[np.int64], NDArray[np.float64]],
                           np.histogram(samples, bins=16))
    print(hist, hist.dtype)
    print(bin_edges, bin_edges.dtype)
    bin_edges = (bin_edges[:-1] + bin_edges[1:]) // 2
    fig, ax = plt.subplots()
    ax.grid(True)
    ax.set_xticks(np.arange(0, 8500, 1000))
    ax.set_yticks(np.arange(0, 11, 1))
    ax.set_xlim(0, 8500)
    ax.set_ylim(0, 11)
    ax.tick_params("both", labelsize=12)
    ax.set_xlabel("メモリ使用量 [KiB]", fontsize=14)
    ax.set_ylabel("度数", fontsize=14)
    ax.plot(bin_edges, hist, ".-")
    fig.savefig(f"out/{TH}-median.png", bbox_inches='tight', pad_inches=0)
    fig.savefig(f"out/{TH}-median.pdf", bbox_inches='tight', pad_inches=0)


def main() -> None:
    parent_dir = Path(f"../workdir/log/{TH}")
    files: map[Path] = map(lambda p: p / "mem.csv", parent_dir.glob("*"))
    samples = list(map(mean_by_one_instance, files))
    plot_hist(samples)

    arr = np.array(samples)
    print(arr.mean(), cast(np.float64, np.median(arr)))
    answer = solve(list(map(int, samples)))
    print(answer)


if __name__ == "__main__":
    main()
