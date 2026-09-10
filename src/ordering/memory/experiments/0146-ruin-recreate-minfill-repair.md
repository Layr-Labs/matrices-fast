# 0146 — Ruin-recreate with greedy min-fill repair (VOL-RR series)

## Hypothesis

The shipped five-descent owns every 5-window exactly, but strided
16/32/64/128-windows destroyed and rebuilt by greedy min-fill escape the
basins it converges to. Repair rule (not ruin geometry) is the live axis:
min-degree reconstruction replays the incumbent dead, min-fill finds new
basins. Terminal seat keeps upstream stages crown-identical (no cascade
starvation by construction).

## Diff in spirit

- New `rgreedy::ruin_window_reconstruct` (deterministic strided ruin,
  greedy min-fill rebuild via live deficiency, lowest-index ties,
  TripleWork budget, `Some` iff differs; no RNG anywhere).
- Terminal call site: gate `n in (32,1000]`, `nnz ≤ 60k` (later 5k),
  16 hill-climbed attempts at 4M ops, best-of exact, single strict admit.
- En route: fixed own Game-API bug (missing `reset()` → empty livelist
  panic on row 1).

## Ablation path (one variable each, all on 74b6ccd, rebaselined 0.793834)

| step | change | result |
|---|---|---|
| degree repair | min-degree rebuild | 0 movers — basins converged, replays dead |
| min-fill repair | deficiency rebuild | sporttournament18 −4.6 (−0.01) — FIRES |
| 8→16 attempts | coverage ×2 | +1 row (−0.03) |
| k=64 scale | attempt%3 16/32/64 | waterund11 −196, 4/0, −0.55 — scale live |
| k=128 | attempt%4 +128 | +mtg1b but +33 stg5 cascade regression (mid seat) |
| terminal seat | same, finished recipient | 3/0 (−0.16), regression gone — seat theory confirmed |

## Result

- Terminal form: SCORE 0.793834 → **0.793818 (−0.16)**, 3/0/297
  (korcns −33.1, sporttournament18 −19.4, mtg1b −16.7), halves −0.25/−0.08
  same sign, worst 0.688 vs 0.682 (slow rows structurally excluded),
  suite 118/118.
- Submitted as `79556ca5` (terminal seat, nnz ≤ 60k, uncharged repair):
  **FAILED hidden 2 s cap** (~98 s in). Autopsy: ~deg×w deficiency evals
  (~8k at k=128), none charged — tens of millions of unbudgeted word-ops
  on a dense hidden row. Ledger units ≠ wall time, 2nd dated witness.
- Fixed form VOL-RR2 (nnz ≤ 5000 — all winners ≤4404; every eval charged):
  identical −0.16/movers. Submitted as `386a14c2`, validating at write
  time.

## Why it won or lost

Min-fill repair works where min-degree replays because deficiency sees
fill the degree sequence hides; scale works because 5-windows are owned
and 16–128-windows are not; terminal seat works because mid-pipeline
wins starve completion re-polish (cascade lesson, priced). The kill came
from the cost model, not the generator: uncharged sub-quadratic work
inside a millisecond-claimed ledger.

## Links

- Log lines: VOL-RR series, 79556ca5 kill, VOL-RR2 submit.
- Techniques: ruin-recreate (new page owed if the line promotes);
  ledger-units lesson (2nd witness).
- Closed-by-this-page: min-degree window repair; mid-pipeline RR seat.
- Live: RR2 verdict; degeneracy candidate (ticket B); peo below-anchor
  law (33/33, structural, unshippable alone).
