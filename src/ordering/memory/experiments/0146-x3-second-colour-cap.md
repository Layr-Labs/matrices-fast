# 0146 — x3, the untested second-colour cap

## Question
0145's follow-up list leaves "x-sets at further caps (x3, x16) and metric α
variants untested". The second-colour block in `indep_first.rs` pushes
`greedy_independent_set_excluding` at caps inf / 15 / 9 / 5 under the iter265a
gate (`n <= 18_000 && nnz <= 80_000`). Cap 3 was never tried.

## Runs (this box, full 300-row dev corpus)

| revision | score | outcome |
|---|---:|---|
| synced tip `62654a5` (submission 3085f82e, hidden 0.843173) | 0.792300 | baseline, fill 0.924373 |
| + x3 at the full x gate | — | **RUN FAILED**: `mpbp_35` (n=11120, nnz=40790) exceeded the 2.0 s per-matrix cap |
| + x3 gated to `nnz <= 20_000` | 0.792300 | **exact tie**, fill 0.924373, zero movers |

## Reading
Cap 3 is the *most* expensive member of the family downstream, not the cheapest:
a lower cap keeps more mid-degree vertices in the core, so the lift, the AMD
walk and every metric pass that follows all get bigger. At the full gate that
was enough to kill `mpbp_35` outright. Where it is affordable — `nnz <= 20_000`
— it produces nothing: the dedup on `(xs, pairs)` absorbs it, or the core it
proposes never beats the caps already in the list. Both halves of the family's
range are therefore closed, and x16 is the same bet in the opposite direction
(a higher cap collapses toward `x-inf`, which is already in the list).

REJECTED. No production change retained; the tree is byte-identical to the
synced tip.

## Box note, which matters more than the result
`mpbp_35` sits under the 2.0 s cap on the tip here but goes over with one extra
greedy set — while 0145 records the tip's *worst row* at 1.026 s on its own box.
This box is roughly 2x slower on that row. Every absolute second in this
`memory/` tree is box-relative (already the standing warning in
`open-questions.md`), and on this box the usable timing margin for new work is
much thinner than the log implies. Screen candidates on flops here, but do not
trust a local "worst 1.0 s" as headroom.
