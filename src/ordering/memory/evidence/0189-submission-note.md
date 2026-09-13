# iter26 — the alternate-seed chain keeps the frontier's scope, priced in work

## Context and goal

The benchmark is a fill-reducing ordering challenge: for each matrix the worker
must return a permutation within a **2 s per-matrix wall cap**, and the score is
the weighted mean of per-bucket geometric means of `flops(candidate)/flops(AMD)`
over three size buckets (`lt_1k` 0.30, `1k_10k` 0.30, `gt_10k` 0.40 — lower is
better). The graded run is the same repository harness the contestant runs
locally, so a local pass and the graded pass share both the definition and the
cap constant.

The frontier (`ab30c0e`) has been the accepted best for this line for several
iterations. Everything this session has added — a strictly-accepting terminal
`rgreedy` ladder that draws extra random-restart passes late in the pipeline —
has been accepted on dev (0.792439 → 0.792212, −2.8 relative bips) and then
either killed by the cap remotely or, once, scored.

## Prior work and the exact failure being repaired

Submission `71c2c5fe` was the first of five ladder builds to *complete* a graded
run. It scored **0.843153** against the frontier's **0.842857** — `+2.96e-4`
worse — and was rejected. Its only difference from `6ad8cc5e` (killed at 105 s
into the hidden corpus) is that 0187 had gated the alternate-seed PEO chain off
above `n = 10 000`. The terminal ladder accepts only strict improvements and
runs last, so it can never make a row worse; therefore **all** of the `+2.96e-4`
is attributable to the gate. The lesson is sharp: a stage whose measured dev
yield in a band is *exactly zero* can still carry real hidden value in that
same band, and the gate bought its cap margin by deleting value rather than by
pricing work.

## Hypotheses

1. The chain's *scope* should be the frontier's (50 000), because the hidden
   corpus demonstrably pays in the 10 000-50 000 band.
2. The chain's *cost* should be shaped by work, not by `n`: the chain already
   charges every round `n + nnz + lnnz` against a ledger, so the ledger is a
   work meter. A smaller allowance above 10 000 buys the first seeds' rounds
   wherever a round is cheap and declines to spend where one round costs most
   of the allowance.
3. If the 2 s cap has a non-`order()` component (process spawn, pattern read,
   permutation write), it would be the cheapest possible margin. Test it before
   assuming it away.

## Approach selection

First a census, then the shape. A test-only instrument (`peo_extract::prof`
plus three `#[cfg(test)]` marks in the chain, drained before the chain so the
earlier PEO chains cannot be charged to it) prints per row: `13p.prep` (the
vendored symbolic trio `permute_pattern` + `EliminationTree::from_pattern` +
`column_counts_gnp`), `13p.recon` / `13p.mcs` (our fill-graph rebuild and the
two MCS sweeps over it), `13p.seeds` / `13p.rounds` and `13p.ledger`. Second,
the same marks measure the effect of any shape change on the capped quantity
per row. Third, a standalone bench links the real `ssi_worker_protocol` crate
to price the plumbing.

## Environment and exact commands

```
# build + run the probe (sandboxed, persistent probe target dir)
bash target/probe-sandbox.sh build
SSI_PROBE_PHASES=1 bash target/probe-sandbox.sh run > evidence/0189-probe-<tag>.log
python3 target/probe-diff.py <baseline.log> <candidate.log>      # per-row flops delta

# the plumbing bench (links the workspace's real protocol crate)
rustc -O --edition 2021 /tmp/io_bench.rs \
  --extern ssi_worker_protocol=target/release/deps/libssi_worker_protocol-*.rlib \
  --extern ssi_scoring=target/release/deps/libssi_scoring-*.rlib \
  -L dependency=target/release/deps -o /tmp/io_bench && /tmp/io_bench

# the official local verification (same harness the grader dispatches)
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "<hypothesis>"
```

## Measurements

**The chain's census at the frontier's scope** (`0189-probe-chainscoope-full.log`):
267/300 rows run the chain; Σ `13.alt` = 6.58 s, of which our own fill-graph
kernel is **48 %** (recon 1.78 + mcs 1.37) and the vendored trio 28 %; 3 508
rounds over 1 754 seeds (≈2 rounds per seed — the chain's own `fin == inc` stop
fires long before the 8-round ceiling). The 38 rows with `10 000 < n <= 50 000`
carry **4.40 s of that band's 27.28 s of `order()` (16.1 %), second only to
`1.portfolio` (7.97 s)**. Cost and yield are anti-correlated on dev: the three
beneficiaries cost 0.013 / 0.071 / 0.144 s (`syn40hfsg`, `maxcsp-ehi-85-297-71`,
`mpbp_15`), while the eleven dearest rows (0.15-0.20 s) all yield exactly
0.0000.

**The plumbing control** (`/tmp/io_bench`, real protocol crate): the worker
reads a 15.3 MB pattern file in 0.0088 s and writes the permutation in
0.0030 s at dev's largest row (n = 272 878, nnz = 1 637 268); 0.032/0.007 s at
56 MB (4× that size). The capped quantity is `order()`; there is no free margin
in the plumbing.

## Implementation

`src/ordering/mod.rs`:

* `PEO_ALT_MAX_N` 10 000 → **50 000** (the frontier's own scope).
* new `PEO_ALT_WIDE_N = 10_000`, `PEO_ALT_LEDGER_WIDE = 1_000_000`, and a
  single `ledger_cap` binding used by both of the chain's ledger tests: above
  10 000 units the allowance is 1e6 instead of 4e6. Structural only — `n`, `nnz`
  and the incumbent's own fill; never the matrix's identity, never a clock.

`src/ordering/peo_extract.rs`: the test-only `prof` module (kernel seconds +
call count), no effect on any shipped binary.

## Results (full 300-row dev corpus, production path)

`COUNTS` **300/300 byte-identical** to the full-scope build; `SCORE = 0.792212`
both ways (bucket geomeans 0.8874 / 0.8391 / 0.6856). The shape is dev-neutral
*by construction*, because no dev row above 10 000 has ever had a nonzero chain
yield — which is exactly why dev could not rank it and 0187's gate looked free.

| | full scope (frontier) | this candidate |
|---|---|---|
| Σ `13.alt`, corpus | 6.58 s | 4.02 s (−39 %) |
| Σ `13.alt`, 10 000 < n ≤ 50 000 | 4.25 s | 1.66 s (−61 %) |
| per-row, wide band | 0.112 s mean | 0.044 s mean |
| worst wide-band row (`arki0013`) | 0.180 s | 0.085 s |
| rounds per wide-band row | 8-37 | 4-16 |
| Σ `order()`, corpus | 129.2 s | 127.3 s |

Official local sandboxed harness, 300 matrices with the 2 s cap:
**300/300 OK, score 0.792212, tiebreak 0.924447** (`results.tsv` row
`1789208378`), matching the probe to the printed precision.

## Failures and course corrections

* **Cost-oriented micro-optimization was ruled out by arithmetic, not taste.**
  The instrument shows 48 % of the chain sits in our own kernel, but halving it
  returns only ~0.03 s per wide-band row — an order of magnitude less than the
  0.11-0.16 s the gate had been buying. A bit-identical kernel speedup is still
  worth doing for margin, but it cannot carry this decision.
* **A cheaper draw on the shared rows was rejected by measurement.** Pricing the
  ladder's draw down to its sparse rung (5e7 ops) on the `10 000 < n <= 12 000`
  rows — the only rows where the draw and the chain's wide-band allowance share
  a row — costs 0.14 bips: `powerflow0300p` 292481 → 293009
  (`0189-probe-tightladder.log`, one regressed row, zero improved). The 5e7 rung
  does not find that mover, so the ladder keeps its 2e8 draw there.
* **The local cap kills are host contention, not the change.** Three builds of
  this session were killed locally on rows whose in-process `order()` is
  0.22-0.43 s; the clean 300/300 pass above came from the same code on a quieter
  host, and the timing-sensitive evidence here is read in-run (per-row marks),
  not by differencing two runs.

## Caveats

Dev cannot rank shapes inside the wide band — every shape there is
dev-neutral — so only the hidden receipt can accept or reject this one. The
hidden corpus is known to contain rows in that band that the frontier's own
pipeline leaves improvable (that is what `71c2c5fe` measured), and it is known
to contain rows whose `order()` sits within ~0.05 s of the 2 s cap (four ladder
builds died there and the same code minus the band passes). This candidate
trades `0.044 s` per wide-band row (down from `0.112 s`) for keeping the chain
present on all of them, with 4-16 of the rounds the frontier had.

## Next steps

1. Read the receipt: if it completes, the delta against 0.842857 measures the
   chain's band value at one-third spend; if it is killed, the killer lies in
   the wide band and the allowance needs a second, tighter tier.
2. Independent of the receipt: the instrument says 48 % of the chain is our own
   `Vec<Vec<u32>>` fill-graph rebuild plus two MCS sweeps — a CSR rewrite is
   bit-identical by construction and would buy the same margin on every row
   without touching what any stage considers.
