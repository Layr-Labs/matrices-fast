# Index

The map of the knowledge base. One line per page, grouped by type. Read this
first; keep it current whenever you add, rename, or retire a page.

## Current best
- Best score: **0.871827** (weighted geomean flop ratio vs AMD; fill tiebreak
  0.960774), dev corpus **300** matrices, 2026-09-02
  ([0005](experiments/0005-relabelled-amf-multistart.md), relabelled-AMF
  multi-start). Previous: 0.876925 / fill 0.962248, 2026-07-26.
- Per bucket: lt_1k 0.9063 (147) · 1k_10k 0.8991 (108) · gt_10k 0.8255 (45).
- **The graded corpus is NOT this corpus.** The same tree that scores 0.876925 on
  dev graded **0.898117** on the hidden eval corpus. Both numbers are real; they are
  different corpora. Never quote a dev score as a graded prediction, and prefer
  changes whose mechanism is structural over changes sized to dev's magnitudes.
- **A submission must STRICTLY beat the frontier.** A 0.00% diff is *rejected*, not
  merely left unpromoted — verified the hard way (submission `dedbfbea`, a
  deliberately score-neutral documentation+probe ship, graded 0.898117 = frontier
  and was rejected). Score-neutral work has to ride along with a scoring win.
- Current `src/ordering/` approach: a **best-of portfolio** in `mod.rs` — ~30
  candidate orderings (feral AMD/AMF variants, METIS/Scotch/KaHIP, plus
  hand-rolled RCM / Sloan / ND / GGGP / MinFill) **and budgeted relabelled-AMD and
  relabelled-AMF multi-starts**, each scored with feral's own `Σ cⱼ²` and the
  cheapest returned, anchored on the grader's exact AMD so the ratio can never
  exceed 1.0. See [best-of-portfolio](techniques/best-of-portfolio.md).
- **Timing headroom is the binding constraint,** but noisier than earlier pages
  claimed: repeat runs of the same probe on the same code vary **~1.6×**, so the
  local worst case is good to one significant figure only. Currently **0.439 s**
  (`arki0016`) against the 2.0 s SIGKILL, up from 0.384 s before
  [0005](experiments/0005-relabelled-amf-multistart.md). NOTE those two numbers
  come from a box roughly 2.5× faster than the one the older 0.9–1.0 s figures on
  this page were recorded on — compare timings only within one box, and treat the
  earlier absolute numbers as history. The old "grader is 3-5× slower than local"
  rule is provably false — see
  [0003](experiments/0003-relabelled-amd-multistart.md). Use the comparative rule
  instead: stay at or below the worst case of a revision known to have passed.
  Measure with `probe_timing_and_score` before adding anything.
- **The search policy of the relabel multi-start is settled: uniform i.i.d. is
  optimal at fixed cost.** 17 explore/exploit policies swept, none robustly better;
  see [0004](experiments/0004-structured-relabelings.md). Do not re-derive it.
  Its constructive corollary is the current best: you cannot aim one lottery, so
  **run a second one on a different objective** —
  [0005](experiments/0005-relabelled-amf-multistart.md).
- See the latest entry in [log.md](log.md).

## Tooling
- [`../probe.rs`](../probe.rs) — TEST-ONLY measurement module (`#[cfg(test)]`,
  never shipped, never read by the grader). The harness prints `(capped)` instead
  of a time, so this is the only way to see the cap. Run:
  `cargo test --release -- --ignored --nocapture --test-threads=1 probe_<name>`
  - `probe_timing_and_score` — per-matrix `order()` time + the current score.
  - `probe_ties` — the matrices still tied at AMD (the target list).
  - `probe_family` — cost AND benefit of each candidate variant, separately.
  - `probe_large` — what the big matrices, gated out by `n` caps, would gain.
  - `probe_relabel_amd` — relabelled-AMD at FLAT restart counts (4/8/16/24).
  - `probe_relabel_budget` — relabelled-AMD under a per-matrix time BUDGET;
    reports score AND true combined worst case for each `(budget, cap)`. This is
    the one that chose the shipped policy.
  - `probe_relabel_search` — relabelled-AMD SEARCH POLICIES at a FIXED restart
    count (i.e. at identical cost): explore/exploit split x perturbation strength
    x schedule. Scores the pure relabel family against AMD, so policy differences
    are not masked by the rest of the portfolio, and needs no timing measurement
    (cost-neutrality is structural). ~10 s for the whole corpus. It also reports
    **robustness columns** — the advantage on two disjoint corpus halves, and with
    the single biggest-contributing matrix dropped. Use those before believing any
    delta on this corpus; see [0004](experiments/0004-structured-relabelings.md).

> **Package matters.** `src/ordering/` compiles only into `ssi-candidate-worker`,
> so a probe command without `-p` (or with `-p matrices-fast`) matches ZERO tests
> and exits green, looking like a pass. Use:
> `cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_<name>`

## Literature
_(papers — one note each; see [literature/_TEMPLATE.md](literature/_TEMPLATE.md))_
- _none yet — start from the references in the repo README._

## Techniques
_(algorithm families & primitives — see [techniques/_TEMPLATE.md](techniques/_TEMPLATE.md))_
- [best-of-portfolio.md](techniques/best-of-portfolio.md) — **the architecture**: why
  the AMD anchor makes every candidate free upside, and why the real problem is
  the time budget. Read this first.
- [amd.md](techniques/amd.md) — the anchor (score 1.00 by definition); strong on dense KKT.
- [nested-dissection.md](techniques/nested-dissection.md) — the separator family; in the
  portfolio via METIS/Scotch/KaHIP plus two hand-rolled variants.

## Experiments
_(hypotheses run against the corpus — see [experiments/_TEMPLATE.md](experiments/_TEMPLATE.md))_
- [0000-identity-baseline.md](experiments/0000-identity-baseline.md) — the starter stub; reference point, not competitive.
- [0001-amd-quotient-graph.md](experiments/0001-amd-quotient-graph.md) — AMD port; matched the baseline. **Superseded**: that hand-rolled `amd.rs` is no longer in the tree.
- [0002-measured-gates-metis-kahip.md](experiments/0002-measured-gates-metis-kahip.md) — measure the cap, then buy candidates with the slack. 0.888132 → **0.883906**. WIN. (Its 1.019 s timing figure is ±1.6×; corrected in 0003.)
- [0003-relabelled-amd-multistart.md](experiments/0003-relabelled-amd-multistart.md) — `AMD(Q A Qᵀ)` composed back through `Q` as a randomized-restart minimum degree, on a per-matrix time budget. 0.883906 → **0.876925**. WIN, the largest single gain so far. (Its "wins land in the first handful of restarts" is corrected by [0004](experiments/0004-structured-relabelings.md): true on average, false for the tail wins that carry the score.)
- [0004-structured-relabelings.md](experiments/0004-structured-relabelings.md) — hill-climbing / structured `Q` instead of i.i.d. `Q`, at equal cost. 17 policies swept. **NEGATIVE**: nothing beats i.i.d. robustly; every apparent win is one matrix (`chp_shorttermplan2d`) and flips sign across corpus halves. Closes the top open question. Adds `probe_relabel_search` and the robustness columns.
- [0005-relabelled-amf-multistart.md](experiments/0005-relabelled-amf-multistart.md) — 0004's constructive corollary: relabel + **AMF** (min-fill) as a second multi-start beside relabelled AMD (min-degree). 0.876925 → **0.871827**. WIN, 36 better / **0 worse** / 264 identical, wins in all three buckets, survives both corpus halves and drop-top-5. Worst `order()` 0.384 → 0.439 s. Generalises: *any* ordering routine that reads the input numbering becomes a randomized-restart algorithm under `relabel`, for free.

## Open questions
- [open-questions.md](open-questions.md) — the research queue.
