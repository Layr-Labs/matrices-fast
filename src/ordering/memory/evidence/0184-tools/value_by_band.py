import os, sys, statistics, math

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from timing_audit import load

EV = 'src/ordering/memory/evidence/'
W = [0.30, 0.30, 0.40]
NAMES = ['lt_1k', '1k_10k', 'gt_10k']


def bkt(n):
    return 0 if n < 1000 else (1 if n < 10000 else 2)


def dens_band(n, nz):
    d = nz / n
    return 0 if d < 3 else (1 if d < 12 else 2)


DB = ['sparse<3', 'mid3-12', 'dense>=12']


def score(rows):
    ls = [0.0] * 3
    cnt = [0] * 3
    for k, (t, n, nz, r) in rows.items():
        b = bkt(n)
        ls[b] += math.log(r)
        cnt[b] += 1
    g = [math.exp(ls[b] / cnt[b]) for b in range(3)]
    return sum(W[b] * g[b] for b in range(3)) / sum(W), g, cnt


def main():
    base, _ = load(EV + '0151-probe-baseline-ab30c0e.log')
    print('frontier score', round(score(base)[0], 6))
    for log in ('0175-budget-20m.log', '0175-budget-50m.log', '0175-budget-100m.log',
                '0184-probe-repro-a.log', '0182-probe-seedA-1x2e8.log', '0180-one-rung-5e8.log'):
        rows, lad = load(EV + log)
        sc, _g, _c = score(rows)
        # value by density band, only rows the gate touched
        gated = {k for k, (fn, pre, cnt) in lad.items() if cnt > 0}
        price = {k: rows[k][0] - lad[k][1] for k in gated if k in rows}
        print(f'\n== {log}  SCORE={sc:.6f}  bips={(0.792436-sc)*1e4:.2f}  gated={len(gated)}  sumPrice={sum(price.values()):.1f}')
        print(f"   {'density band':12s} {'rows':>5s} {'movers':>6s} {'sumP':>6s} {'meanP':>7s} {'maxP':>7s}")
        for d in range(3):
            ks = [k for k in gated if dens_band(rows[k][1], rows[k][2]) == d]
            if not ks:
                continue
            mv = [k for k in ks if abs(rows[k][3] - base[k][3]) > 1e-12]
            ps = [price[k] for k in ks]
            print(f"   {DB[d]:12s} {len(ks):5d} {len(mv):6d} {sum(ps):6.1f} {statistics.mean(ps):7.4f} {max(ps):7.4f}   movers={','.join(mv[:4])}")
        # value by n band
        for b in range(3):
            ks = [k for k in gated if bkt(rows[k][1]) == b]
            if not ks:
                continue
            ps = [price[k] for k in ks]
            mv = [k for k in ks if abs(rows[k][3] - base[k][3]) > 1e-12]
            print(f"   bucket {NAMES[b]:8s} rows={len(ks):3d} movers={len(mv):2d} sumP={sum(ps):5.1f} meanP={statistics.mean(ps):.4f} maxP={max(ps):.4f}")


if __name__ == '__main__':
    main()
