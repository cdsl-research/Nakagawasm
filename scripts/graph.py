from __future__ import annotations
import pandas as pd
import matplotlib
from matplotlib.font_manager import FontProperties
import matplotlib.pyplot as plt

font_path = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"
font_prop = FontProperties(fname=font_path)
matplotlib.rcParams["font.family"] = font_prop.get_name()
plt.rcParams["font.size"] = 14

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
plt.savefig(f"data/{name}.png")
plt.savefig(f"data/{name}.pdf")
plt.close("all")
