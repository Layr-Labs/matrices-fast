# 0150 — Remove the three narrow stage-1b force-adoption windows, and nothing else

- **Date:** 2026-09-10
- **Score:** 0.792300 → **0.792439** (weighted geomean flop ratio vs AMD, lower
  is better; −1.75 relative bip, i.e. a small loss)
- **Status:** compliance change, measured loss on dev, submitted as a
  single-variable probe

## Hypothesis

Two questions at once.

1. **Compliance.** The stage-1b adoption rule inherited from the promoted tree
   force-adopted the independent-set lift inside three narrow windows —
   `400..=1000`, `1800..=2500`, and `8_000..20_000 && nnz >= 50_000` — each
   named in its own source comment after the dev-corpus family it was fitted
   around (`digabel`, `hydro`, `mpbp_35`). Those select on instance identity,
   not on structure. On an evaluation corpus disjoint from dev they fire on
   rows chosen at random with respect to the property that motivated them, so
   they are removed regardless of what dev says.
2. **Isolation.** A tree carrying this removal *together with* a stage-13
   retirement and a `best_flops` write-back repair was SIGKILLed by the 2 s
   per-matrix cap on the graded corpus, twice, while the tree it descends from
   passes the same corpus three times byte-identically. This record isolates
   the window removal so the grader can say whether it is the change that costs
   the cap.

## What changed

`src/ordering/mod.rs`, one condition at the stage-1b adoption site plus one new
constant. The three windows are deleted; the `n >= 20_000` force-adoption is
kept and named `INDEP_FORCE_MIN_N`, because it is an ordinary monotone
predicate on `n` and not a window fitted around particular rows. The margin
rule (`INDEP_IMMEDIATE_MARGIN = 9/10`) is untouched, and so is the deferred
path. Nothing else in the tree is changed.

## Result

Dev, 300 patterns, `cargo run --release --offline --locked`:

| bucket | count | flop geomean |
|---|---|---|
| `lt_1k` | 147 | 0.887516 |
| `1k_10k` | 108 | 0.839747 |
| `gt_10k` | 45 | 0.685651 |
| **weighted** | 300 | **0.792439** |

Fill tiebreak 0.924472.

## Why it lost on dev

The windows were fitted to dev rows, so removing them gives back exactly the
dev value that fitting them bought — that is the definition of the trade, and
1.75 relative bip is the price. What it buys is that the rule now generalises:
above `INDEP_FORCE_MIN_N` the residual core after the independent-set lift is a
mesh-like Schur complement that the downstream chain polishes well, while the
portfolio incumbent on such a row has usually received little more than AMD.
That statement is about structure and is testable on any corpus; "n is between
400 and 1000" is not.

Note that the removal changes which candidate is the incumbent entering stages
2–15 on a quarter of the dev corpus (74 of 300 rows, including 21 of 45
`gt_10k` rows). Downstream stage *cost* is basin-sensitive, so a change that
reads as a speed-up on every dev row it touches can still cost time on rows dev
does not contain. That is the reason this record exists as an isolated
submission rather than as one hunk inside a larger commit.

## Follow-ups

- If this tree clears the cap, the cap regression in the larger commit is
  **not** here, and the remaining two changes (stage-13 retirement,
  `best_flops` write-back repair) should be bisected the same way.
- If it does not clear the cap, the adoption rule is the site, and a principled
  replacement for the vacated population — a structural predicate rather than a
  size window — has to pay for its own downstream cost.

## Links

- Techniques: independent-set-first lift, [0143](0143-independent-set-first-lift.md),
  [0145](0145-second-colour-class-metric-cores.md)
