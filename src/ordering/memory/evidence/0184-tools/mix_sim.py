import os, sys, math, statistics

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from timing_audit import load

EV = 'src/ordering/memory/evidence/'
W = [0.30, 0.30, 0.40]
LOGS = {
    '20m': '0175-budget-20m.log',
    '50m': '0175-budget-50m.log',
    '100m': '0175-budget-100m.log',
    '200m': '0184-probe-repro-a.log',
    '200mB': '0182-probe-seedB-1x2e8.log',
    '500m': '0180-one-rung-5e8.log',
}


def bkt(n):
    return 0 if n < 1000 else (1 if n < 10000 else 2)


def band(n, nz):
    d = nz / n
    return 0 if d < 3 else (1 if d < 12 else 2)


def score(rows):
    ls = [0.0] * 3
    cnt = [0] * 3
    for k, (t, n, nz, r) in rows.items():
        b = bkt(n)
        ls[b] += math.log(r)
        cnt[b] += 1
    g = [math.exp(ls[b] / cnt[b]) for b in range(3)]
    return sum(W[b] * g[b] for b in range(3)) / sum(W), g


def main():
    base, _ = load(EV + '0151-probe-baseline-ab30c0e.log')
    runs = {}
    for tag, f in LOGS.items():
        rows, lad = load(EV + f)
        price = {k: rows[k][0] - lad[k][1] for k, (fn, pre, cnt) in lad.items() if cnt > 0 and k in rows}
        runs[tag] = (rows, price)
        sc, g = score(rows)
        print(f'{tag:6s} SCORE={sc:.6f} bips={(0.792436-sc)*1e4:5.2f} gated={len(price):3d} '
              f'meanP={statistics.mean(price.values()):.4f} maxP={max(price.values()):.4f} sumP={sum(price.values()):5.1f}')

    def mix(b_sparse, b_mid, b_dense):
        rows = {}
        price = []
        for k, (t, n, nz, r) in base.items():
            b = band(n, nz)
            tag = (b_sparse, b_mid, b_dense)[b]
            if tag is None or k not in runs[tag][0]:
                rows[k] = (t, n, nz, r)
                continue
            rows[k] = runs[tag][0][k]
            if k in runs[tag][1]:
                price.append(runs[tag][1][k])
        sc, g = score(rows)
        movers = sum(1 for k in base if abs(rows[k][3] - base[k][3]) > 1e-12)
        regress = sum(1 for k in base if rows[k][3] > base[k][3] + 1e-12)
        return sc, movers, regress, price

    print()
    print(f"{'config (sparse,mid,dense)':30s} {'SCORE':>9s} {'bips':>6s} {'mv':>3s} {'reg':>3s} {'sumP':>6s} {'meanP':>7s} {'maxP':>7s}")
    cfgs = [('100m', '200m', '200m'), ('50m', '200m', '200m'), ('20m', '200m', '200m'),
            ('100m', '200m', '500m'), ('50m', '200m', '500m'), ('50m', '100m', '200m'),
            ('100m', '100m', '200m'), ('200m', '200m', '200m'), ('100m', '100m', '100m'),
            ('50m', '50m', '200m'), ('50m', '50m', '500m'), ('20m', '50m', '100m')]
    for c in cfgs:
        sc, mv, rg, price = mix(*c)
        print(f"{str(c):30s} {sc:9.6f} {(0.792436-sc)*1e4:6.2f} {mv:3d} {rg:3d} "
              f"{sum(price):6.1f} {statistics.mean(price):7.4f} {max(price):7.4f}")


if __name__ == '__main__':
    main()
