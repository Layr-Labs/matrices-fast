# 0093 — METIS shape variants on the dense mid band

**Date:** 2026-09-07. **Base:** `017a036` (my 0092, hidden 0.85328, dev 0.814072).
**Result:** dev **0.811703** (−23.7 bips), changed rows 1 (pooling_sppa9tp 0.4417 → 0.1625); every other row bit-identical; worst
same-box `order()` 1.204 s vs 1.208 s (2-vCPU pod, min-of-2).
**Hidden:** submission `5d2aeab9` graded **0.85328 = the base, 0.00 %, REJECTED** — not one
hidden row in the band responds to a METIS shape. A one-dev-row lottery ticket does not
translate; keep the family (it is 75 ms on rows far from the cap and the old challenge's
`pooling_*tp` twins did respond) but never spend a submission on such a change alone.

## Finding

`probe_census` on the rows where the SSI tree still beat 0092 (every generator
alone, ratio and seconds): on `pooling_sppa9tp` (n 5040, nnz 121k, density 24)
METIS with a different SHAPE is the best ordering the pipeline ever sees —
`max_imbalance` 0.05 → 0.2512, 0.02 → 0.2167, 0.10 + `niparts` 16 → 0.1801, each
~25 ms — against 0.5047 for default METIS, 0.4550 for the best non-partitioner
generator and 0.4417 for the finished pipeline. The cascade never reaches those
variants there: they are gated `nnz < 60k` and behind `part_extra2`, which only
opens when the DEFAULT separator beats the min-degree incumbent. The old
challenge's notes recorded the same `imb = 0.05` winner on this family's
`pooling_*tp` twins (sppc3tp 0.20, sppc1tp 0.42).

## Shipped

Three fixed shapes (0.05; 0.02; 0.10 with 16 initial partitions), queued after
the cascade with the other ported families, gated `n <= 30k`, `nnz >= 20 n`,
`60k <= nnz < 250k`. Finished-pipeline result on the winner: 0.4417 → **0.1625**.
The row's total time rises 0.44 → 0.85 s (the chains now fire on a much better
incumbent), still under the tree's slowest row.

## Not shipped (measured)

- A fixed-α5 relabelled-AMF pass for the seeds the cycled-α loop gives other
  α values (census: best single generator on procurement1large and
  chp_shorttermplan2d): **zero** changed rows, +3 s corpus time.
- METIS seeds 2/21/26/55 on the band: win only where the shapes win.
- The same census on every other dense row (sppa9pq, sppb5pq, sppc1pq, kissing2,
  meanvar-orl400, maxcsp-ehi, qapw, gams05, nuclear10a/104, methanol400): no
  METIS variant beats the incumbent. One dev winner; 25 ms a ticket.
