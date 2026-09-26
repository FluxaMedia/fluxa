#!/usr/bin/env python3
import argparse, json, statistics, subprocess, sys, time

p = argparse.ArgumentParser(description="Interleaved torrent startup benchmark against live swarms")
p.add_argument("--build", action="append", required=True, metavar="NAME=BINARY")
p.add_argument("--magnets", required=True, help="file with one 'name<TAB>magnet[<TAB>file_index]' per line")
p.add_argument("--rounds", type=int, default=10)
p.add_argument("--timeout", type=float, default=120)
p.add_argument("--out", help="append raw results as JSON lines")
a = p.parse_args()

builds = [b.split("=", 1) for b in a.build]
magnets = []
for line in open(a.magnets):
    if line.strip() and not line.startswith("#"):
        parts = line.rstrip("\n").split("\t")
        magnets.append((parts[0], parts[1], parts[2] if len(parts) > 2 else "0"))

results = {}
out = open(a.out, "a") if a.out else None
for r in range(a.rounds):
    order = builds if r % 2 == 0 else builds[::-1]
    for name, magnet, index in magnets:
        for build, binary in order:
            try:
                run = subprocess.run([binary, magnet, index], capture_output=True, text=True, timeout=a.timeout)
                res = json.loads(run.stdout.strip().splitlines()[-1])
            except (subprocess.TimeoutExpired, IndexError, json.JSONDecodeError):
                res = {}
            results.setdefault((name, build), []).append(res)
            print(f"round {r + 1} {name:<16} {build:<10} " + " ".join(f"{k}={v:.1f}" for k, v in res.items()) or "failed", file=sys.stderr)
            if out:
                out.write(json.dumps({"time": time.time(), "round": r, "magnet": name, "build": build, **res}) + "\n")
                out.flush()

def summary(vals, n):
    if not vals:
        return f"{'-':>7} {'-':>7}"
    vals = sorted(vals + [a.timeout] * (n - len(vals)))
    p90 = vals[min(len(vals) - 1, round(0.9 * (len(vals) - 1)))]
    return f"{statistics.median(vals):>7.1f} {p90:>7.1f}"

print(f"\n{'magnet':<16} {'build':<10} {'meta med':>8} {'p90':>7} {'first med':>9} {'p90':>7} {'16MB med':>8} {'p90':>7} fails")
for name, _, _ in magnets:
    for build, _ in builds:
        runs = results.get((name, build), [])
        cols = [summary([x[k] for x in runs if k in x], len(runs)) for k in ("meta", "first", "mb16")]
        fails = sum(1 for x in runs if "first" not in x)
        print(f"{name:<16} {build:<10} {cols[0]} {cols[1]}  {cols[2]}  {fails}/{len(runs)}")
