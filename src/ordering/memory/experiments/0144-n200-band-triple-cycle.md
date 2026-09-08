# 0144 — 3-cycle on 100≤n≤200 nnz≤2k after a77d6de FAIL

- **Date:** 2026-09-08
- **Score:** crown 0.804873 → **0.804869** (−0.04 bip). 0.887817 / 0.842581 / 0.714375.
- **Status:** win locally, submitted. Ablation of `a77d6de` (n≤200 3-cycle, FAIL).

## Hypothesis

`a77d6de` failed hidden with 3-cycles on every n≤200 row (~94 public). pooling_adhya4tp (n=170, nnz=936) is the public mover (20581→20540). If the FAIL was timing on the whole small band, a tight n/nnz gate that still contains pooling should keep the scoring claim at ~1/4 the public fires. If the FAIL was the 3-cycle neighbourhood itself, this will fail too.

## What changed

`src/ordering/mod.rs` only. After crown four-restart SmallScore admit:

- 3-cycles (768 draws, both directions) only when `100 <= n <= 200 && nnz <= 2_000`.
- ~23 public fires vs 94 for a77d6de.
- pooling_adhya4tp in; maxcsp-langford (n=660) out; cap rows out.

Not shipped (measured 0-delta or timing poison this loop): leftover-gain four2, SS-winner insert, n-band insert+reverse.

## Result

pooling_adhya4tp 20581→**20540**. Same public score as a77d6de. n<100 3-cycles were 0 extra public. 1k_10k/gt_10k bit-identical to crown. Worst isolated: maxcsp **1.56 s**, lee4_09 **0.997 s**.

## Census that drove the gate

- leftover four/triple/pair: **0 public exact wins** on n≤1000.
- four2 after leftover or after SmallScore: 0-delta.
- insert+reverse on SmallScore winners: one maxcsp 1229-flop sliver, 6-digit score unchanged, worst 1.70 s.
- insert+reverse on this n-band: pooling stayed 20581.

## Follow-ups

- On FAIL: 3-cycles even in a 23-row band fail hidden; do not retry 3-cycles.
- On REJECT thin: do not widen n; leftover exact is dead on public nnz≤12k.
- On PROMOTE: next increment still n≤1000, not 3-cycle / leftover n>1000 / extra etree / nnz>30k LNS / all-n SmallScore lottery.
