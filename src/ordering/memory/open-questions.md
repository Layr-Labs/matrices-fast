# Open questions

The research queue: leads worth chasing, gaps in the knowledge base, and
hypotheses not yet tested. Add a line whenever you notice one; resolve it by
linking to the page (experiment, technique, or literature note) that answers
it, rather than deleting it — a resolved question is a useful signpost.

## Active

- [ ] **Price the rest of the pipeline by score-per-millisecond.**
      [0146](experiments/0146-stage13-funds-lee-metric-cores.md) did this for
      stage 13 (alternate-seed PEO) and found 0.19 bips of score defended by
      0.10 s on the corpus's slowest rows — the time was worth ~9 bips spent
      elsewhere. No other stage has ever been priced this way. `9.reduce`
      (13.24 s corpus-wide) and `1.portfolio` (30.36 s) are the obvious next
      candidates. Two rules from 0146: `phase_mark` reports `best_flops`, which
      stages after 11 leave stale, so a stage's apparent flatness proves
      nothing — A/B the FINAL per-row ratio; and re-price periodically, because
      stage 13's recorded wins (mpbp_34/35, arki0013, gabriel09) had silently
      stopped reproducing as later stages grew to reach the same completions.

- [ ] **Work the residual core, not the full matrix (0062 substrate).** [0075](experiments/0075-residual-core-minfill.md) tested four-seed residual-core AMF/AMD relabels (no movers) and exact MinFill for `cn <= 1,000` (7 movers, now shipped). Still untested: multi-depth prefixes K in {2,4,5} (distinct cores only), and late phases run on the core under a per-call WORK budget as a REPLACEMENT of a late phase (never additive on rows >= 0.8 s). See [0062](experiments/0062-reduce-then-amf-terminal.md).
- [ ] **Relabel the OTHER numbering-sensitive routines (top lead).**
      [0005](experiments/0005-relabelled-amf-multistart.md) established the general
      form: *any* ordering routine whose output depends on the input vertex numbering
      becomes a randomized-restart algorithm under `relabel`, for the cost of one pass
      and with zero score risk under the best-of floor. Two objectives are now
      relabelled (AMD, AMF). Never relabelled: the hand-rolled RCM, Sloan, `nd_order`
      / `ndfm_order` (their BFS-median and GGGP separator choices both read the
      numbering), and MinFill. Prefer the ones whose objective differs MOST from
      min-degree, since that difference is where the second lottery's prizes came
      from. Cost per family is `RELABEL_BUDGET/nnz` passes, so price each with
      `probe_family` before adding it.
       [0020](experiments/0020-medium-exact-search.md) tested one fixed relabeling
       for RCM, both Sloan weights, `nd_order`, and `ndfm_order`: all five produced
       zero wins. [0075](experiments/0075-residual-core-minfill.md) repeated the
       test with eight deterministic seeds under the production gates and again
       found zero wins; do not add these families without new evidence.
- [x] **RESOLVED (positive) — Conditional search escalation on below-anchor matrices.**
      Answered by [0060](experiments/0060-conditional-search-escalation-below-anchor.md):
      Substitutive re-tiering on `best_flops < amd_flops` promoted at hidden **0.869723**.
      The additive extra-pass draft timed out on a hidden matrix and must not be retried on `n >= 10k`.
- [x] **RESOLVED (positive) — Scale leftover search by the AMD-anchor margin, not by size.**
      Answered by [0061](experiments/0061-margin-scaled-leftover-search.md):
      Miss-retry first subtree round + well-below exact LNS / relabel tickets scored
      0.843658 → **0.843358** (−3.00 bip). Raising first-round `max_s` on a *successful*
      first round loses (0.843829). Widening k5/k4 to `n <= 12k` and adding
      `adjacent_triple_descent` convert nothing. `gt_10k` leftover nnz 60k–100k did not
      move at four digits.
- [ ] **Sweep the relabelled-AMF `dense_alpha`.** Shipped at α=5.0 only (the base AMF
      candidate's α). α ∈ {0.5, 2.0, 2.5} is the same argument one level down — a
      different α is a different objective, hence another distinct lottery — and it is
      cheap inside the existing gate. Mirror the base AMF α sweep in `order()`.
- [ ] **Is `RELABEL_AMF_MAX_NNZ = 130_000` leaving anything above it?** The ceiling is
      a cost bound, not a measured optimum. Measure the 130k–400k band's AMF per-pass
      cost in ISOLATION (`probe_family`) before raising it; the dev corpus has few
      matrices there, so the honest expectation is a small score gain against a real
      cap risk. Measure first.
- [ ] **Does the budget want to be non-uniform across buckets?** The shipped
      `RELABEL_BUDGET` spends the same ~0.3 s everywhere, but `gt_10k` carries
      weight 0.40 over only 45 matrices (~4.4× the per-matrix leverage of
      `lt_1k`). A bucket-weighted budget — more restarts where a win is worth
      more — was never tested. Note `n` is known inside `order()`, so this stays
      a pure function of `(n, nnz)`.
- [x] **RESOLVED (negative) — The big tied matrices are gated out of everything.**
      Answered by [0039](experiments/0039-tie-breaker-battery-negative.md): they are
      gated out for good reason. Nested dissection on these KKT graphs is 2.2x-4.5x
      WORSE than AMD, not merely unaffordable (`faclay75` METIS ratio 2.2273 at
      14.7 s; `gabriel10` 4.4925; Scotch on `faclay75` returns 9519x; KaHIP 38-48 s).
      `probe_large` measured all of them. Do not widen the partitioner gates.
      Original text follows.
- [ ] ~~**The big tied matrices are gated out of everything.**~~ `faclay75`
      (n=272878), `acopf_case9241pegase_qcqp` (n=313068), `gabriel10` (n=244056),
      `unitcommit_200_100_1_mod_8` (n=146830) all tie at 1.000 and receive only
      AMD plus at most one AMF pass, because the candidate gates are capped on
      `n`. But cost tracks nnz, not n, so some may have unused budget —
      `acopf_case9241pegase_qcqp` gets literally nothing but the baseline. These
      are the highest-leverage matrices on the corpus (gt_10k weight 0.40 over
      only 45 matrices). `probe_large` is written to measure exactly this.
- [ ] **How fast is the grader, really?** Partly answered and partly reopened by
      [0003](experiments/0003-relabelled-amd-multistart.md): the header's "3-5×
      slower than local" claim is false (a 1.019 s local revision passed), and
      repeat local runs vary ~1.6×, so we are tuning against a number we know to
      one significant figure. Nothing in the harness output exposes grader
      timing. Until it does, the only defensible rule is comparative — stay at or
      below the worst case of a revision known to have passed.
- [ ] **How much of the remaining headroom is even measurable on 300 matrices?**
      [0004](experiments/0004-structured-relabelings.md) showed that one `gt_10k`
      matrix is worth ≈0.002 of score, so any change smaller than that is
      indistinguishable from luck on this corpus, and the hidden eval corpus is
      refreshed per round. Nothing currently tells us the *variance* of the score
      under corpus resampling. A bootstrap over the 300 dev matrices (resample with
      replacement, re-aggregate) would give the confidence interval that says which
      past "wins" in this log were real — cheap to write, and it changes how every
      future result should be read.
- [ ] Do any ML/RL-guided ordering ideas fit a stdlib-only, deterministic,
      2 s/matrix `order()`? Survey the literature before assuming yes/no.
- [ ] The hand-rolled `nd_order` / `ndfm_order` use a plain **degree sort** at
      their leaves (`ND_LEAF=200`, `NDFM_LEAF=100`) and for unsplittable
      separators. The textbook hybrid hands leaves to minimum degree instead.
      Cheap to try (AMD on the induced subgraph) — but note their gate is nearly
      a subset of the METIS gate, so the upside may be small.

## Resolved

- [x] *"Structured relabelings, not random ones (was the top lead)."* **Answered NO
      by [0004](experiments/0004-structured-relabelings.md).** At a fixed restart
      count, no explore/exploit policy beats uniform i.i.d. draws: 17 policies swept
      (split ratio × perturbation strength × decay/reset/no-chain schedules), and
      every policy whose full-corpus score looked better flipped sign between
      disjoint corpus halves and lost to i.i.d. once one matrix
      (`chp_shorttermplan2d`) was dropped. Chaining — the part that makes it a hill
      climb — contributes nothing, and bigger perturbations beat smaller ones
      monotonically, so the relabeling→flops map has **no exploitable local
      structure**: AMD's tie-breaking is a global cascade, and the family is a pure
      lottery. Do not retry with an RCM- or partitioner-seeded `Q`; the evidence is
      against the mechanism, not against one perturbation. The only lever that
      reliably improves this family is **more restarts**, which is a timing problem
      (see the monotone budget sweep in
      [0003](experiments/0003-relabelled-amd-multistart.md)).
- [x] *"Where is the real headroom — is it nested dissection on the larger
      families?"* Partly answered by
      [0002](experiments/0002-measured-gates-metis-kahip.md): a 12-variant
      partitioner sweep (METIS/Scotch/KaHIP seeds, imbalance, ND→AMD switch,
      dense-quotient) improved only **7 of 260** matrices. Partitioner-parameter
      tuning is near its ceiling; the headroom is not there.
- [x] *"What density threshold should gate an expensive path? Measure, don't
      guess."* Measured — cost tracks **nnz**, not n (`qapw`, n=705/nnz=87k,
      costs 0.539 s; matrices 300× larger cost less). Per-variant costs are
      tabulated in [0002](experiments/0002-measured-gates-metis-kahip.md); use
      `probe_family` to extend the table rather than guessing a new gate.
- [x] *"Port the demo ND+AMD hybrid's exact-MD inner loop to a quotient-graph
      MD."* Obsolete as written: the portfolio now calls library METIS/Scotch/
      KaHIP, all of which already do multilevel ND with an AMD base case, and
      none breach the cap under their gates.

- [ ] **Is the `lt_1k` subtree chain exhausted?** [0038](experiments/0038-subtree-chain-into-lt1k.md)
      opened the bucket and took it 0.8965 → 0.8952 with 17 movers, but **55 ties
      remain** there and only ONE reallocation was tested (`max_blocks 8` x
      `budget 4M`). Sweep `max_blocks`/`budget`/`max_s` inside the fixed 32M
      ceiling, and try a third stream on the `n <= 1_000` exact search. `lt_1k`
      worst is 0.824 s against a 1.72 s corpus worst, so the headroom is real.
- [ ] **Does `SUBTREE_MIN_N` want to go below 64?** 70 dev matrices have `n < 100`.
      200 → 64 was worth only 0.7 bip, so the curve is flattening, but it was never
      pushed to 16 or 32. Cheap to test; bound the setup cost, not just the search.
- [ ] **Re-measure the base on every new box before trusting any timing page.**
      The same frontier tree measures 0.829 s (0025's box) and 1.702 s (0026's box).
      Every absolute second in `memory/` is box-relative. A revision judged safe on
      a fast box can be at 85% of the cap on a slow one — which is the most likely
      mechanism behind the three hidden-cap failures in 0025.

## Measured follow-up on frontier 996e8d6

Production-core stage-6 polishing at the 10M starting-fill estimate is below its screen: 2 final winners / 0.07338 dev bips ([0089](experiments/0089-production-core-stage6-screen.md)). Most improved cores lose to other completed pipeline candidates. Distinct core portfolios remain open; the exhibited blend718 portfolio win receives no proposal sets in this polish screen.

The 100k transplant with verification reservation yields 26 winners / 0.93139 dev bips, retaining every winner when gated below raw AMD and reducing paid rows 196→152 ([0090](experiments/0090-transplant-verification-reservation-screen.md)). It misses the predeclared three-bip floor. Reserving final verification before spending the last donor allowance is useful bookkeeping; larger ledgers or different donor choices remain separate experiments with unvalidated runtime exposure.
