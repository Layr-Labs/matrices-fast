# 0185 — Density-shaped terminal ladder (sparse band priced down)

Submission: `yukon submit` receipt appended to
[0162](../evidence/0162-remote-submission-ledger.txt). Evidence:
`0185-probe-density-shaped.log`, `0186-local-harness-density-shaped.log`,
`score.json`, `results.tsv` (row `1789202595`).

## Change

`src/ordering/mod.rs`, terminal stage only, inside the existing
`n <= 12_000 && nnz <= 200_000` window:

```
budget = 50_000_000  if nnz <  3 * n     // the sparse band
budget = 200_000_000 otherwise           // mid and dense
```

same seed (`0x9E37_79B9_7F4A_7C15`), same strict acceptance, nothing upstream of
the terminal stage touched. The predicate is structural (`nnz`, `n`) and selects
nothing by name, identity, or corpus membership.

## Why

[0184](0184-timing-resolution-and-in-run-draw-price.md) priced the draw in-run for
the first time: the price is ~0.00027 s per 1e6 ops, but the constant is
density-dependent (sparse 0.315 ns/op, mid 0.275, dense 0.150). The sparse band
is therefore where the draw is dearest per op — 66 rows at 0.0629 s mean /
0.0957 s max for 2e8 — and it is also where the corpus's worst per-row add lives.

Splicing the per-row deterministic results (`COUNTS` flops are byte-identical
across runs of one build) of `0175-budget-50m` for the sparse rows onto this
build's own results predicted the sparse band loses nothing measurable: all three
of its movers are still produced. The built-and-measured result:

| build | dev SCORE | vs frontier (`ab30c0e` 0.792436) | price mean | p90 | max | corpus add |
|---|---|---|---|---|---|---|
| flat one 2e8 (shipped iter23) | 0.792215 | −2.21 bips | 0.0537 | 0.0733 | 0.0957 | 12.8 s |
| **density-shaped (this)** | **0.792212** | **−2.24 bips** | **0.0414** | 0.0717 | **0.0902** | **9.9 s** |

Both axes move the right way: +0.03 dev bips and −23 % mean / −25 % corpus added
time, with the sparse band's 66 rows 0.0629 → 0.0166 s mean and 0.0957 → 0.0375 s
max. Exactly three rows change flops versus the flat build
(`rsyn0830m04m` 171801 → 171821, `rsyn0820m04m` 152992 → 152996,
`rsyn0840m02m` 45077 → 45000 — the last one more than pays for the other two).
Zero rows regress against the frontier (strict acceptance).

## Verification

* probe (test-only, production path): `SCORE = 0.792212`, worst `order()`
  1.168 s (`0185-probe-density-shaped.log`);
* official sandboxed local harness, full 300-row dev corpus: **300/300 OK**,
  `score.json` 0.792212, fill 0.924447, `results.tsv` row `1789202595`
  (`0186-local-harness-density-shaped.log`), matching the probe exactly;
* `cargo test` baseline suite green (the harness runs it before scoring).

## Cap context that motivated it

The iter23 submission of the flat build, `436d52d2` (commit `88de718`), was
**also killed at the cap**: Actions run `34683124829`, scored run starts
08:26:37.43, `RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap`
at 08:28:24.47 = **107 s**, against 114 s (iter16, two ungated 2e8 draws) and
112 s (iter20, tiered). The three kills sit in the same 99-114 s band as ten
failed benchmark jobs from five other solvers, although the per-row add of the
three builds differs by 2-4×: at that resolution the kill position is a property
of the corpus/grader, not of the ladder's price. What is within reach is the
price itself, and this build spends 25 % less of it for a slightly better dev
score. Kill census: `0162` ledger.
