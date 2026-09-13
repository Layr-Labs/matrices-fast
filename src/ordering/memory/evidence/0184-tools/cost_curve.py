import glob, os, statistics, sys, math

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from timing_audit import load


def score_of(path):
    s = w = None
    for l in open(path, errors='replace'):
        if l.startswith('SCORE ='):
            s = float(l.split('=')[1])
        if l.startswith('WORST order()'):
            w = float(l.split('=')[1].split('s')[0])
    return s, w


def main():
    base, _ = load('src/ordering/memory/evidence/0151-probe-baseline-ab30c0e.log')
    files = sorted(glob.glob('src/ordering/memory/evidence/01*.log'))
    print(f"{'log':44s} {'SCORE':>9s} {'WORST':>6s} {'gate':>4s} {'sumP':>6s} {'meanP':>6s} {'p90P':>6s} {'maxP':>6s} {'movers':>6s} {'bips':>7s} {'bp/s':>6s}")
    for f in files:
        rows, lad = load(f)
        if not rows or len(rows) < 300:
            continue
        sc, w = score_of(f)
        if sc is None:
            continue
        price = []
        for name, (full_n, pre, cnt) in lad.items():
            if cnt == 0 or name not in rows:
                continue
            price.append((rows[name][0] - pre, name))
        if not price:
            continue
        ps = [p[0] for p in price]
        movers = 0
        bips = (0.792436 - sc) * 1e4
        for k in base:
            if k in rows and abs(rows[k][3] - base[k][3]) > 1e-12:
                movers += 1
        regress = sum(1 for k in base if k in rows and rows[k][3] > base[k][3] + 1e-12)
        print(f"{os.path.basename(f):44s} {sc:9.6f} {str(w):>6s} {len(price):4d} {sum(ps):6.1f} "
              f"{statistics.mean(ps):6.4f} {sorted(ps)[int(.9*len(ps))]:6.4f} {max(ps):6.4f} {movers:6d} {bips:7.2f} "
              f"{bips/max(sum(ps),1e-9):6.3f}  regress={regress}")


if __name__ == '__main__':
    main()
