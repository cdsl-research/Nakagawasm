from __future__ import annotations

import japanize_matplotlib
import matplotlib.pyplot as plt
import pandas as pd

japanize_matplotlib.japanize()

name = "cwasm"
name = "cnative"

df: pd.DataFrame = pd.read_csv(f"data/{name}.txt",
                               header=None,
                               names=("datetime", "RSS"))

df["RSS"] /= 1024

print(df.head())

plt.figure()
# df.head(200).plot()
df.head(200).plot(figsize=(6.4 * 0.9, 5.0))

plt.grid()
plt.legend().remove()
plt.ylim(0, 10_0)
plt.yticks()
plt.xlabel("経過時間 (sec)")
plt.ylabel("RSS (MB)")
plt.savefig(f"data/{name}.png", bbox_inches='tight', pad_inches=0)
plt.savefig(f"data/{name}.pdf", bbox_inches='tight', pad_inches=0)
plt.close("all")
