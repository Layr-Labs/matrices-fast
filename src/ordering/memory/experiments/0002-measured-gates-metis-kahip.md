# 0002 — Measure the cap, then buy candidates with the slack

**Date:** 2026-07-26
**Score:** 0.888132 → **0.883906** (fill 0.966671 → 0.965241), 300-pattern dev corpus
**Status:** WIN, committed

## The idea

The portfolio in `order()` is anchored on the grader's own AMD, so every extra
candidate is free upside and the *only* question is what fits in the 2 s cap
(see [best-of-portfolio](../techniques/best-of-portfolio.md)). But nobody had
measured the cap — the harness prints `(capped)` instead of a time, and the
module header asserted a worst case of 0.313 s. So: measure first, then spend.

Added [`src/ordering/probe.rs`](../../probe.rs), a `#[cfg(test)]` module (never
compiled into the shipped binary, never read by the grader) that times `order()`
per matrix and does what-if scoring of candidates before they are wired in.

## What the measurements said

**1. The header was wrong by 3×, and the margin is thin.**

| | secs | n | nnz |
|---|---|---|---|
| `crudeoil_lee4_10` | **1.019** | 17809 | 120632 |
| `ringpack_30_2` | 1.011 | 17999 | 121458 |
| `nuclear104` | 0.869 | 39098 | 257806 |
| `faclay75` | 0.721 | 272878 | 1379706 |

Against a 2.0 s SIGKILL that is a ~2× margin, and one breach FAILs the whole
run. **The slow tier cannot absorb any new candidate.** Cost tracks nnz, not n:
`qapw` (n=705, nnz=87496) costs 0.539 s, more than matrices 300× larger.

**2. A blanket multi-start is a trap.** Running 12 partitioner variants (METIS
seeds / imbalance / ND→AMD switch / dense-quotient, Scotch seeds, KaHIP
seeds+modes) on everything below n=30k/nnz=60k improved **only 7 of 260**
matrices, and cost up to **2.449 s extra** on a single matrix — an instant FAIL.
Whole-portfolio score effect: −0.0042.

**3. But the 7 wins are cheap if bought individually.** Timing each variant
separately (`probe_family`) showed the value is concentrated and the expensive
variants are separable from the cheap ones:

| variant | max secs | wins | winner on |
|---|---|---|---|
| METIS `imb 0.05/0.10` | 0.054 | 3 | maxcsp-langford-3-11 0.450→0.392 |
| METIS `switch 100/400` | 0.068 | 4 | ndcc13 0.745→0.721, nuclear25a 0.697→0.683 |
| METIS `seed 21` | 0.054 | 2 | multiplants_mtg1b 0.782→0.775 |
| KaHIP `seed 2` | 0.237 | 2 | **mpbp_34 0.567→0.452** |
| KaHIP `Eco` | 0.411 | 3 | **mpbp_35 0.588→0.469**, chimera_selby 0.811→0.691 |
| METIS `dense_quotient` | 0.056 | **0** | — (dropped) |
| KaHIP `Strong`, Scotch seeds | 0.408 | few, dominated | — (dropped) |

## The change

Two blocks, each gated where the measurement says it is free:

* **METIS shape variants**, `n < 30_000 && nnz < 60_000`. Note these vary the
  *shape* of the dissection (where separators fall, where ND hands off to
  minimum degree), whereas every METIS candidate already in the portfolio varied
  only the *amount of work* (`niparts`, `fm_passes`). Five variants add ≤0.285 s;
  worst combined `order()` in the envelope is 0.668 s.
* **KaHIP seed 2 + Eco**, `n < 12_000 && nnz < 45_000` — much tighter, because
  KaHIP costs up to 0.65 s at n≈22k. The gate is drawn just above the three
  wins; worst combined `order()` in the envelope is 0.823 s.

Both stay under the pre-existing 1.019 s worst case, so **the global worst case
is unmoved** and the safety margin is exactly what it was.

## Result

Predicted 0.883906 from the probe; measured 0.883906 from the harness — the
what-if scoring is exact, which makes the probe trustworthy for future
candidates. Per bucket: lt_1k 0.9091→0.9080, 1k_10k 0.9129→0.9114, gt_10k
0.8538→**0.8452** (the KaHIP wins both land in the heaviest bucket).

## What this says about where to go next

The portfolio is near the ceiling of *partitioner-parameter* tuning: 12 variants
moved 7 matrices. 122 of 300 matrices still tie AMD at exactly 1.000, and they
resist every separator/profile/bandwidth candidate tried so far. The next gain
needs a qualitatively different candidate.

The lead queued for next session (untested — the probe was written but the run
was cut short, see [open-questions](../open-questions.md)): **relabelled-AMD
multi-start**. AMD's output depends on its tie-breaking, and its tie-breaking
depends on the vertex NUMBERING — so running feral's own AMD on `B = P A Pᵀ` for
a fixed pseudo-random `P`, then composing back through `P`, is a genuinely
different minimum-degree ordering for the cost of one AMD pass, without writing
an MD implementation. That is the one thing not yet tried on the set of matrices
where *AMD itself* is the winner. `probe_relabel_amd` in
[`probe.rs`](../../probe.rs) is written and compiles; it reports the score at 4 /
8 / 16 / 24 restarts and the per-matrix cost. Run it first.

## Links
- [best-of-portfolio](../techniques/best-of-portfolio.md) — the architecture and the cost budget.
- [amd.md](../techniques/amd.md) — why AMD is so hard to beat on this corpus.
