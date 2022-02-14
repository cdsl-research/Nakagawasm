from __future__ import annotations

import numpy as np
import pandas as pd
from scipy import stats
from pathlib import Path
from matplotlib import pyplot as plt
import japanize_matplotlib

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
    return df["uss"].median()


def plot_hist(samples: list[float]) -> None:
    plt.hist(samples, bins=16)
    plt.grid()
    plt.xlim(5000, 9000)
    plt.tick_params("both", labelsize=12)
    plt.xlabel("メモリ使用量の中央値の分布 (kBytes)", fontsize=14)
    plt.ylabel("個数", fontsize=14)
    plt.savefig(f"{TH}-median.png", bbox_inches='tight', pad_inches=0)
    plt.savefig(f"{TH}-median.pdf", bbox_inches='tight', pad_inches=0)


def main() -> None:
    parent_dir = Path(f"../workdir/log/{TH}")
    files: map[Path] = map(lambda p: p / "mem.csv", parent_dir.glob("*"))
    samples = list(map(mean_by_one_instance, files))
    plot_hist(samples)
    answer = solve(samples)
    print(answer)


if __name__ == "__main__":
    main()
