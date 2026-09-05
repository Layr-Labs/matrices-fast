# Index

The map of the knowledge base. One line per page, grouped by type. Read this
first; keep it current whenever you add, rename, or retire a page.

## Verified 2026-09-05 checkpoint

- Current shippable locally verified candidate score: **0.843657** (fill **0.943813**) on all 300 dev matrices, 2026-09-05. Base `ea67ff8` score 0.843978. Conditional search escalation on below-anchor matrices: lt_1k **0.890328** / 1k_10k **0.865965** / gt_10k **0.791924** (−3.21 bips gain). Worst same-machine order() 1.352 s (faster than base 1.358 s). See [0060](experiments/0060-conditional-search-escalation-below-anchor.md).
- Preceding four-pivot/atomic winner: dev **0.84419540581772**, hidden **0.870307**, officially promoted as `da03dc2c` / source `649c230`.
- New component-factored five-pivot cleanup with cache and boundary repairs: dev **0.8440714862418132**, fill **0.943946**; all **300 trusted cases and 44 active tests pass**. Exact counts agree with the isolated probe. **42 better / 2 worse / 256 unchanged** versus the preceding winner; both corpus halves and drop-five sensitivity improve. Official follow-up result pending submission/validation.
- [0059: component five-pivot cleanup](experiments/0059-component-five-pivot-cleanup.md) records the exact formula, native implementation, work ledger, negative controls and source audits.
- [0057: exact four-pivot cleanup](experiments/0057-exact-four-pivot-cleanup.md) preserves the prior winner. [0056: exact triple cleanup](experiments/0056-exact-triple-cleanup.md) preserves its predecessor and the subsequent hidden timeout.
- Logical work caps do not prove total wall time. Observed isolated k5 worst **0.970 s**, preceding winner **0.987 s**; timing varies with machine load.
- Development and hidden scores measure different corpora. Older current-best entries below are historical and may lag their source.

## Historical current-best entries
- Best shippable locally verified candidate score: **0.849251** (fill
  **0.947647**) on all 300 dev matrices, 2026-09-03. On top of promoted
  submission `26932eb`, cap the first two medium-tier subtree rounds at
  `max_s=256` and cap only the first round at 12 blocks. Per bucket:
  **0.893893 / 0.874387 / 0.796916**; the small and large buckets are exact
  frontier controls. The direct worst call is 1.079 s versus 1.081 s for the
  promoted source on this machine. See
  [0042](experiments/0042-medium-first-round-block-cap.md).
- Best locally verified candidate score: **0.850370** (weighted geomean flop
  ratio vs AMD; fill tiebreak **0.948420**), dev corpus **300** matrices,
  2026-09-03 ([0036](experiments/0036-multiround-cascading-terminal-subtree-refinement.md):
  multi-round cascading terminal subtree refinement with sparsity-gated large tier).
  Current official promoted hidden score: **0.875942** (submission `e4a98396`, commit `1417f26`).
- Per bucket: lt_1k 0.896482 (147) · 1k_10k **0.875531** (108) ·
  gt_10k **0.796916** (45).
- Best locally verified candidate score: **0.849801** (weighted geomean flop
  ratio vs AMD; fill tiebreak **0.947880**), dev corpus **300** matrices, 2026-09-03
  ([0038](experiments/0038-subtree-chain-into-lt1k.md): the bounded subtree chain
  extended into `lt_1k`, stacked on
  [0035](experiments/0035-chained-terminal-subtree-refinement.md)).
  Base for 0036 is frontier `1417f26` (submission `e4a98396`, hidden
  **0.875942**), which measures **0.850370** on this box.
- Per bucket: lt_1k **0.895059** (147) · 1k_10k 0.875531 (108) ·
  gt_10k 0.796916 (45). `lt_1k` had been frozen at 0.8965 across 0021-0025 and
  0035; 0036 is the first change to move it.
- **THIS PAGE LAGS THE CODE — RE-RUN THE BASE, DON'T READ IT.** At commit
  `1417f26` this block still described 0035 (0.850464 / 0.875665 / 0.797049)
  while the `mod.rs` committed beside it was already 0036's tree, which measures
  **0.850370 / 0.875531 / 0.796916**. A stale block silently inflates or deflates
  whatever delta the next session claims against it. The score is a
  deterministic, hardware-independent function of (pattern, permutation), so
  always probe the unmodified base first.
- **TIMING CALIBRATION IS PER-BOX AND THE SPREAD IS LARGE.** The frontier tree
  that 0025 measured at "worst 0.829 s" measures **1.702 s** on the 2026-09-03
  box — ~2x slower. Absolute seconds on pages 0002-0035 are NOT comparable to
  page 0036. Always re-measure the base on the current box before judging a
  revision's timing, and compare only within one run series.
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
- [0040](experiments/0040-terminal-small-exact-cascade.md) — Reallocate the
  small subtree budget to `max_s=256`, 16 blocks x 2M, then run deterministic
  whole-graph exact search after the complete promoted pipeline, with a second
  salted parallel round conditioned on a strict first-round win. **0.849801 →
  0.849309** locally, but submission `1fbb1a08` **FAILED private validation**.
  The additive terminal work was removed and must not be retried.
- [0041](experiments/0041-medium-subtree-block-cap.md) — Isolate `max_s=256` to
  `1000 <= n < 10000`. A global cap helps medium but hurts large; the bucket
  gate preserves the public gain. **0.849487 → 0.849194**, with small and large
  buckets unchanged, but submission `fd357537` **FAILED hidden timing**. Direct
  worst-case runtime rose from 1.081 s to 1.606 s.
- [0042](experiments/0042-medium-first-round-block-cap.md) — Retain medium
  `max_s=256` but cap its first round at 12 blocks after diagnosing 0041's exact
  GitHub Actions failure. **0.849487 → 0.849251**, fill **0.947647**, worst direct
  call **1.079 s**; full trusted 300-matrix run passes. The 8-block and 750k
  alternatives are negative controls.
- [0049](experiments/0049-bounded-medium-terminal-cascade.md) — A fourth bounded
  medium-window variant still exceeded the hidden 2 s cap. **CLOSED**: do not
  retry the smaller-window family.
- [0050](experiments/0050-late-round-budget-step.md) — On the accepted all-8M
  chain, raise only conditional rounds 4 and 5 to 16M. Dev **0.846054**, hidden
  **0.871827**, fill **0.955667**. **PROMOTED TO #1** as `28d9a9d2` / `e93779c`.
- [0051](experiments/0051-round4-budget-step.md) — Raise only round 4 from its
  accepted 16M to 32M and keep round 5 at 16M. Dev **0.845707**, hidden
  **0.871418**, fill **0.955486**. **PROMOTED TO #1** as `d6de8499` / `7177486`.
- [0052](experiments/0052-round4-64m-boundary.md) — Raising round 4 globally
  from 32M to 64M improved dev to **0.845411**, but submission `de541fe9`
  exceeded the hidden 2-second matrix cap. Global 64M is closed.
- [0053](experiments/0053-selective-lower-medium-round4-depth.md) — Reclaim the
  score-positive part of 0052 by using 64M only for `1,000 <= n < 6,000` and
  retaining hidden-proven 32M elsewhere. Dev **0.845469**, fill **0.944729**;
  1k-10k worst observed call **0.661 s**. Submitted for hidden validation.

## Open questions
- [open-questions.md](open-questions.md) — the research queue.
