# 0189 — The capped asset priced in work: the chain keeps the frontier's scope and pays a *work-shaped* allowance

Direction: **stop shaping the chain's gate by `n` and shape it by what actually
prices a round.** 0187 removed the alternate-seed chain above `n = 10 000` as a
"time-negative exchange" — the chain is 5.8 % of the corpus's phase time and its
measured dev yield above 10 000 is exactly zero. The hidden corpus rejected that
trade (submission `71c2c5fe`: the only difference from the 105 s kill
`6ad8cc5e` is that gate, and it completed at 0.843153 against the frontier's
0.842857 = **+2.96e-4 worse**, and the terminal ladder is strictly accepting, so
every bit of that is the gate). This experiment re-prices the same stage from
the *other* side: keep the scope, cut the allowance in the currency the chain
itself spends.

## Instrument: the chain's own price census (test-only, never shipped)

`peo_extract::prof` + three marks in the chain (`#[cfg(test)]`, drained first so
`12.peo` and the large-instance chain cannot be charged to `13.alt`) print, per
row, inside the existing `PHASES` line:

| mark | meaning |
|---|---|
| `13p.prep` | `permute_pattern` + `EliminationTree::from_pattern` + `column_counts_gnp` per round |
| `13p.recon` / `13p.mcs` | `candidates_bounded`: the `Vec<Vec<u32>>` fill-graph rebuild vs the two MCS sweeps over it |
| `13p.seeds` / `13p.rounds` | seeds entered / rounds attempted |
| `13p.ledger` | ledger units charged |

Full corpus, chain at the frontier's 50 000 scope (`0189-probe-chainscoope-full.log`):

* 267/300 rows run the chain; Σ `13.alt` = **6.58 s**, of which `13p.prep` 1.85
  (28 %), `13p.recon` 1.78 (27 %), `13p.mcs` 1.37 (21 %) — i.e. **48 % of the
  stage is our own fill-graph kernel**, the rest is the vendored symbolic trio
  and the candidate scoring.
* 3 508 rounds over 1 754 seeds (≈2 rounds per seed: the chain's own
  `fin == inc` stop fires long before the 8-round ceiling).
* On the 38 rows with `10 000 < n <= 50 000` the chain is **4.40 s of that
  band's 27.28 s of `order()` = 16.1 %**, second only to `1.portfolio` (7.97 s):
  the band's dearest rows are `transswitch0300p` 0.154 s, `arki0013` 0.180,
  `powerflow0300p` 0.147, `mpbp_48` 0.139, `mpbp_35` 0.136, `popo200` 0.131.
* Cost and yield are anti-correlated on dev: the 3 beneficiaries cost
  0.013 / 0.071 / 0.144 s (`syn40hfsg`, `maxcsp-ehi-85-297-71`, `mpbp_15`),
  while the 11 dearest rows (0.15-0.20 s) all have **exactly zero** yield.

## The negative control: the cap has no non-`order()` half

Finding 14 of this session established that the 2 s cap wraps the whole
sandboxed worker subprocess. A standalone bench linking the *real*
`ssi_worker_protocol` crate (`/tmp/io_bench`, same code path as the worker)
measures the non-`order()` share of that subprocess:

| n | nnz | pattern file | worker `read_pattern` | `write_permutation` |
|---|---|---|---|---|
| 272 878 | 1 637 268 | 15.3 MB | 0.0088 s | 0.0030 s |
| 500 000 | 3 000 000 | 28.0 MB | 0.0057 s | 0.0012 s |
| 1 000 000 | 6 000 000 | 56.0 MB | 0.0322 s | 0.0066 s |

≈0.006 s at dev's largest row and 0.039 s at 4× it: the capped quantity is
`order()` by construction, so every second of cap margin has to come out of the
search, not the plumbing.

## Change

`src/ordering/mod.rs`:

* `PEO_ALT_MAX_N` 10 000 → **50 000** — the chain's scope goes back to the
  frontier's own (`71c2c5fe`'s measured hidden cost).
* new `PEO_ALT_WIDE_N = 10_000`, `PEO_ALT_LEDGER_WIDE = 1_000_000`: above
  10 000 the chain charges its rounds against a **1e6-unit allowance** instead
  of 4e6. The ledger is `n + nnz + lnnz` per round, i.e. a *work* meter, so the
  wide band buys the first seeds' rounds wherever a round is cheap and refuses
  the rows where one round costs most of the allowance. Nothing is keyed to the
  matrix's identity, only to `n`, `nnz` and the incumbent's own fill.

## Measured (probe, production path, full 300-row dev corpus)

`0189-probe-wideworkbudget.log` vs `0189-probe-chainscoope-full.log`:

* `COUNTS` **300/300 byte-identical**, `SCORE = 0.792212` both ways (bucket
  geomeans 0.8874 / 0.8391 / 0.6856): the shape is dev-neutral **by
  construction**, because no dev row above 10 000 has ever had a nonzero chain
  yield. Dev therefore cannot rank shapes here — only the hidden receipt can.
* Σ `13.alt` 6.58 → **4.02 s** (−39 %); below 10 000 unchanged (2.27 → 2.30).
* The wide band 4.25 → **1.66 s** (−61 %), i.e. 0.112 → **0.044 s per row**;
  worst band row `arki0013` 0.180 → 0.085, `transswitch0300p` 0.154 → 0.050,
  `powerflow0300p` 0.147 → 0.050, `mpbp_34` 0.168 → 0.042.
* Rounds per band row 8-37 → 4-16; the ledger lands at 0.6-1.0e6 where the full
  allowance was spent in full.
* Corpus Σ `order()` 129.2 → 127.3 s (the candidate is the frontier's chain
  scope plus the ladder, minus 2.6 s of band spend).

## Rejected in the same run (measured, not assumed)

Pricing the ladder's draw down to its sparse rung (5e7) on the `10 000 < n <=
12 000` rows — the only rows where the draw and the chain's wide-band allowance
share a row — **loses 0.14 bips**: `powerflow0300p` 292481 → 293009
(`0189-probe-tightladder.log`, the single regressed row). The 5e7 rung does not
find that mover, so the ladder keeps its 2e8 draw there.

## Official local sandboxed harness

`evidence/0189-local-harness-wideworkbudget.log` + `results.tsv` row
`1789208378` — **300/300 OK, score 0.792212, tiebreak 0.924447**, agreeing with
the probe to the printed precision, after the same run had cap-killed three
earlier builds of this session on unrelated rows (host contention, not the
candidate).
