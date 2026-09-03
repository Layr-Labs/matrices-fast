# Index

The map of the knowledge base. One line per page, grouped by type. Read this
first; keep it current whenever you add, rename, or retire a page.

## Current best
- Best locally verified candidate score: **0.848955** (weighted geomean flop
  ratio vs AMD; fill **0.947530**), dev corpus **300** matrices, 2026-09-03
  ([0041](experiments/0041-size-tiered-block-cap.md): the block-size cap tiered
  by graph size). Base is our own promoted frontier `344a5d2`
  (submission `26932eba`, hidden **0.874601**), which measures 0.849487 here.
- Per bucket: lt_1k 0.893893 (147) · 1k_10k **0.873403** (108) ·
  gt_10k 0.796916 (45).
- **`max_s` (the searched-block size cap) is the highest-yield knob found so far,
  and it is a FUNCTION OF GRAPH SIZE, not a constant.** One global value cannot
  serve every bucket: lowering the shared cap to 256 improves `1k_10k` while
  HURTING `gt_10k`. Current tiers — `n < 1k` → 256, `1k-10k` → 128, `>= 10k` →
  384. Two sweeps (0040, 0041) took the score 0.849801 → 0.848955 on this knob
  alone, and both were also timing-NEUTRAL-or-better, because a smaller cap does
  less work per block.
- **`lt_1k` TRANSLATES TO THE HIDDEN CORPUS BETTER THAN DEV SUGGESTS.**
  [0038](experiments/0038-subtree-chain-into-lt1k.md) moved dev by 4.24 bips and
  the hidden score by **5.6 bips** — a ~1.32x translation, where the only prior
  calibration point (0023, a `1k_10k`/`gt_10k` change) gave ~0.53x. Dev appears
  to UNDER-weight `lt_1k` relative to the graded corpus, which makes it a better
  place to spend effort than its dev numbers alone imply. Two points, not a law.
- **CHECK ROBUSTNESS BEFORE ACCEPTING ANY SEARCH-CONFIG WIN.** In 0040 the two
  configs nearest the incumbent looked like wins on the full bucket but FLIPPED
  SIGN once their three biggest movers were dropped. Use disjoint halves +
  drop-top-3, per [0004](experiments/0004-structured-relabelings.md). A config
  whose neighbours agree (a plateau) is trustworthy; an isolated spike is not.
- **THIS PAGE LAGS THE CODE — RE-RUN THE BASE, DON'T READ IT.** At commit
  `1417f26` this block still described 0035 (0.850464) while the `mod.rs`
  committed beside it measured 0.850370. Always probe the unmodified base first.
- **TIMING CALIBRATION IS PER-BOX AND THE SPREAD IS LARGE.** The `971649b` tree
  that 0025 measured at "worst 0.829 s" measures **1.702 s** on the 2026-09-03
  box — ~2x slower. Absolute seconds on pages 0002-0036 are NOT comparable to
  pages 0038+. Compare only within one run series.
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
  relabelled-AMF multi-starts**, plus bounded exact elimination-game search on
  small and medium sparse graphs and on ranked elimination-tree subtrees. Each
  is scored with feral's own `Σ cⱼ²` and the
  cheapest returned, anchored on the grader's exact AMD so the ratio can never
  exceed 1.0. See [best-of-portfolio](techniques/best-of-portfolio.md).
- **Timing headroom is the binding constraint,** but noisier than earlier pages
  claimed: repeat runs of the same probe on the same code vary **~1.6×**, so the
  local worst case is good to one significant figure only. The final probe
  measured **0.755 s** (`gams05`) against the 2.0 s SIGKILL; the first candidate
  probe measured 0.843 s, and the synced parent measured 0.776 s on the same
  box. Older timing figures were
  recorded on different hardware — compare timings only within one box, and
  treat earlier absolute numbers as history. The old "grader is 3-5× slower than
  local" rule is provably false — see
  [0003](experiments/0003-relabelled-amd-multistart.md). Use the comparative rule
  instead: stay at or below the worst case of a revision known to have passed.
  The failed 0021 revision measured **0.801 s** locally but timed out on a hidden
  matrix because it requested up to 128M search operations. The bounded 0022
  revision requests at most 32M and measured **0.767–0.777 s** locally. Measure
  with `probe_timing_and_score` before adding anything. The first 0025 attempt
  also timed out on hidden data: its extra 32M phase had a broad
  `n<=350k/nnz<=1.5M` gate. A 16M additive retry inside
  `n<=80k/nnz<=250k` failed with the same timeout. The replacement design
  removes the frontier's 24M terminal pass, substitutes the 16M allocation,
  and measures **0.829 s** locally.
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
  - `probe_tie_breakers` — for every surviving tie with `n >= 1000`, the cost AND
    the achieved ratio of 16 separator/min-fill candidates. Answered 0027.
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
- [0006](experiments/0006-cycled-amf-amd-multistart.md) — Cycled AMF dense_alpha schedule [5.0, 2.0, -1.0, 1.0, 16.0] and alternating AMD aggressive mode. 0.871827 → **0.871434**. WIN (eval 0.889994, promoted).
- [0007](experiments/0007-bucket-weighted-relabel-budget.md) — Dimensional budget scaling ($n \ge 10k \to 500k/36$, $n \ge 1k \to 400k/30$). 0.871434 → **0.870672**. WIN (eval 0.889138, promoted).
- [0008](experiments/0008-relabelled-amf-ceiling-expansion.md) — Raised RELABEL_AMF_MAX_NNZ from 130k to 200k. 0.870672 → **0.870261**. WIN.
- [0009](experiments/0009-robust-amd-envelope-expansion.md) — Raised ROBUST_MAX_NNZ from 130k to 600k for 5 non-aggressive & dense-detection disabled AMD variants. 0.870261 → **0.868096**. WIN (eval 0.888100, promoted).
- [0010](experiments/0010-relabelled-minfill-multistart.md) — Exact deficiency multi-start on $n < 2,000, nnz < 10,000$. 0.868096 → **0.867686**. WIN.
- [0011](experiments/0011-hub-gate-and-floors.md) — Hub-gated restart allocation (`max_deg * 50 <= n`) with mid-band/low-nnz floors + dual-pass independent AMF seeds. 0.867686 → **0.864899**. WIN.
- [0012](experiments/0012-terminal-adjacent-pair-descent.md) — Terminal adjacent-pair descent on exact objective. 0.864899 → **0.864652**. WIN.
- [0013](experiments/0013-terminal-simplicial-promotion.md) — Terminal simplicial promotion on exact dynamic graphs. 0.864652 → **0.864462**. WIN.
- [0014](experiments/0014-custom-quotient-metrics.md) — Custom quotient-graph metrics (SqDiv & SqPure). 0.864462 → **0.863609**. WIN.
- [0015](experiments/0015-small-simplicial-cycled-amd-minfill.md) — Small-graph simplicial promotion, 6-way cycled AMD & scaled minfill. 0.863609 → **0.863272**. WIN.
- [0020](experiments/0020-medium-exact-search.md) — Two bounded serial exact-search stages on `1,000 < n <= 6,000`, `nnz <= 30,000`, followed by pair descent when its existing gate allows it. Synced baseline 0.860780 → **0.859116**. WIN.
- [0021](experiments/0021-exact-subtree-refinement.md) — Exact search over at most 32 ranked, disjoint elimination-tree subtrees with two fixed streams. 0.859116 → **0.851513** publicly, but the hidden run exceeded the 2 s matrix cap. FAILED.
- [0022](experiments/0022-bounded-subtree-work.md) — Cap subtree search at 32 blocks × one stream × 1M requested operations. Accepted-base 0.859116 → **0.852938** publicly; hidden submission pending.
- [0023](experiments/0023-subtree-round-3-chain.md) — Chained subtree round 3 (round=1, 32 blocks, min_s 16, **max_s 512**) after hybridnoise's conditional round 2. Frontier base 0.852246 → **0.851642**. PROMOTED hidden 0.877373 (2026-09-03).
- [0024](experiments/0024-subtree-round-4-chain.md) — Chained subtree round 4 (round=1, 32 blocks, min_s 16, **max_s 768**). 0.851642 → **0.851347**. SUBMITTED (2026-09-03).
- [0025](experiments/0025-adaptive-terminal-deep-subtree-search.md) — Both 32M and 16M additive terminal passes failed the hidden 2 s cap. The lower-work retry replaces the frontier's 24M terminal pass with at most 16M: 4×4M below 10k vertices, 8×2M above. Frontier source 0.851055 → **0.850594**; worst local `order()` 0.829 s (2026-09-03).
- [0038](experiments/0038-subtree-chain-into-lt1k.md) — The subtree chain was gated at `n >= 1_000` from 0021 onward, so the whole `lt_1k` bucket never saw the technique that moved the other two. `SUBTREE_MIN_N = 64`, a reallocated small-graph config (8 deep blocks x 4M = the same 32M ceiling), and a second stream on the `n <= 1_000` exact search. 0.850594 → **0.850167**; 17 better / 0 worse / 283 identical; `lt_1k` 0.8965 → 0.8951 with the other buckets unchanged. WIN.
- [0039](experiments/0039-tie-breaker-battery-negative.md) — 16 separator/min-fill candidates x 31 surviving ties, 496 measurements: **zero wins**, per-candidate minimum ratio exactly 1.0000. METIS on `faclay75` is 2.23x AMD and takes 14.7 s; Scotch returns 9519x; KaHIP 38-48 s. **NEGATIVE**, and it closes the "big tied matrices" open question — the partitioner gates protect the run rather than cost score.
- [0040](experiments/0040-lt1k-block-size-sweep.md) — swept the `lt_1k` small-graph config inside its fixed 32M ceiling. **`max_s` (the cap on searched block size) is the dominant knob and 0038 had it 2x too high**: 512 → 256 takes the bucket 0.894939 → 0.893893, score 0.849801 → **0.849487**, AND drops the corpus worst `order()` 1.774 → 1.610 s. 224/256/288 form a plateau (all robust on disjoint halves and drop-top-3) while 320/384 fail drop-top-3. `min_s` below 16 is a no-op. WIN.
- [0041](experiments/0041-size-tiered-block-cap.md) — the block cap must SCALE WITH GRAPH SIZE. Sweeping the shared `SUBTREE_CFG.max_s` showed 256 helps `1k_10k` (0.8752→0.8742) but HURTS `gt_10k` (0.7969→0.7991) and 512 hurts both, so no single value serves all buckets. Gave `1k_10k` its own `MID_MAX_S`; swept it to **128** (basin 96-192, all robust on halves + drop-top-3). `1k_10k` 0.875176 → **0.873403**, score 0.849487 → **0.848955**; `lt_1k` and `gt_10k` byte-identical. WIN.

## Open questions
- [open-questions.md](open-questions.md) — the research queue.
