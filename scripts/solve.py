from __future__ import annotations
import numpy as np
from scipy import stats


def solve(samples: list[int]):
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
