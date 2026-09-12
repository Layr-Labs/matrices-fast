import statistics, math, sys

def load(path):
    lines = open(path, errors='replace').read().splitlines()
    start = None
    for i, l in enumerate(lines):
        if l.startswith('--- every order()'):
            start = i
            break
    if start is None:
        return None, None
    rows = {}
    for l in lines[start + 2:]:
        p = l.split('\t')
        if len(p) < 5:
            if l.strip() == '' or l.startswith('matrices under'):
                continue
            break
        try:
            t = float(p[0])
        except ValueError:
            break
        rows[p[1]] = (t, int(p[2]), int(p[3]), float(p[4]))
    lad = {}
    cur = None
    for l in lines:
        if l.startswith('LADGATE\t'):
            p = l.split('\t')
            cur = (int(p[2]), float(p[3]), int(p[4])) if len(p) == 5 else (None, float(p[2]), int(p[3]))
        elif l.startswith('COUNTS\t') and cur is not None:
            lad[l.split('\t')[1]] = cur
            cur = None
    return rows, lad

def main(pa, pb):
    A, ladA = load(pa)
    B, ladB = load(pb)
    print('rows A/B', len(A), len(B), 'totals', round(sum(v[0] for v in A.values()), 1), round(sum(v[0] for v in B.values()), 1))
    for tag, rows, lad in (('A', A, ladA), ('B', B, ladB)):
        cost = []
        for name, (full_n, pre, cnt) in lad.items():
            if name not in rows or cnt == 0:
                continue
            t, n, nz, r = rows[name]
            cost.append((t - pre, name, n, nz))
        cost.sort(reverse=True)
        cs = [c[0] for c in cost]
        print(f'--- run {tag}: in-gate rows {len(cost)} price = order() secs - pre-draw secs')
        print(f'    mean {statistics.mean(cs):.4f} median {statistics.median(cs):.4f} '
              f'p90 {sorted(cs)[int(.9*len(cs))]:.4f} max {max(cs):.4f} min {min(cs):.4f} sum {sum(cs):.1f}')
        print('    dearest: ' + ' | '.join(f'{c:.4f}(n={n},nnz={nz})' for c, name, n, nz in cost[:5]))
        print('    cheapest: ' + ' | '.join(f'{c:.4f}(n={n},nnz={nz})' for c, name, n, nz in cost[-4:]))
    both = [k for k in ladA if k in ladB and ladA[k][2] > 0 and ladB[k][2] > 0]
    diffs = []
    for k in both:
        ca = A[k][0] - ladA[k][1]
        cb = B[k][0] - ladB[k][1]
        diffs.append((abs(cb - ca), k, ca, cb, ladA[k][1], ladB[k][1]))
    diffs.sort(reverse=True)
    print('--- per-row draw price A vs B (same rows, same draw) ---')
    print('  rows', len(diffs), 'median |diff|', round(statistics.median(d[0] for d in diffs), 4),
          'p90', round(sorted(d[0] for d in diffs)[int(.9 * len(diffs))], 4), 'max', round(diffs[0][0], 4))
    for d, k, ca, cb, pra, prb in diffs[:6]:
        print(f'    {k:28s} priceA={ca:.4f} priceB={cb:.4f} |d|={d:.4f} preA={pra:.3f} preB={prb:.3f}')
    pairs = [(A[k][1], A[k][2], A[k][0] - ladA[k][1]) for k, (fn, pre, cnt) in ladA.items() if cnt > 0 and k in A]

    def corr(x, y):
        mx = statistics.mean(x)
        my = statistics.mean(y)
        num = sum((a - mx) * (b - my) for a, b in zip(x, y))
        den = math.sqrt(sum((a - mx) ** 2 for a in x) * sum((b - my) ** 2 for b in y))
        return num / den if den else float('nan')

    print('  corr(price,n)=', round(corr([p[0] for p in pairs], [p[2] for p in pairs]), 3),
          ' corr(price,nnz)=', round(corr([p[1] for p in pairs], [p[2] for p in pairs]), 3),
          ' corr(price,nnz/n)=', round(corr([p[1] / p[0] for p in pairs], [p[2] for p in pairs]), 3))
    dense = [p[2] for p in pairs if p[1] / p[0] >= 12]
    sparse = [p[2] for p in pairs if p[1] / p[0] < 12]
    print(f'  dense(nnz/n>=12) n={len(dense)} mean {statistics.mean(dense):.4f} max {max(dense):.4f} | '
          f'sparse n={len(sparse)} mean {statistics.mean(sparse):.4f} max {max(sparse):.4f}')
    sh = [((A[k][0] - ladA[k][1]) / A[k][0], k, A[k][0]) for k in ladA if ladA[k][2] > 0 and k in A and A[k][0] > 0]
    sh.sort(reverse=True)
    print('--- draw share of the row total (run A) ---')
    print('  median', round(statistics.median(s[0] for s in sh), 3),
          'p90', round(sorted(s[0] for s in sh)[int(.9 * len(sh))], 3), 'max', round(sh[0][0], 3))
    for s, k, t in sh[:5]:
        print(f'    {s:.3f} {k:28s} total={t:.3f}')

if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
