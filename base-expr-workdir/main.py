import re
import csv

pat = re.compile(r"(.+Z)\s INFO.+uri: (\S+),\s.*\n")
counter = 0
with (open("event.log") as f, open("event.csv", mode="w", encoding="utf8", ) as g):
    w = csv.writer(g, lineterminator="\n")
    for line in f:
        m = pat.match(line)
        if m[2] == "/login":
            counter += 1
        elif m[2] == "/logout":
            counter -= 1
        w.writerow([m[1], m[2], str(counter)])
