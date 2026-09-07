# Open questions

The research queue: leads worth chasing, gaps in the knowledge base, and
hypotheses not yet tested. Add a line whenever you notice one; resolve it by
linking to the page (experiment, technique, or literature note) that answers
it, rather than deleting it — a resolved question is a useful signpost.

## Active

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

- [x] **"Is stage 6's small-graph polish at a local optimum?"** No - it was at a local
      optimum of 1536 FIXED position tuples. Thirty-two independent stream pairs,
      best-of, are worth 0.86 dev bips through `lt_1k`; sets 129-256 win nothing, so
      the move class itself is now closed ([0085](experiments/0085-stage6-move-sampling-metered-by-fill.md)).
- [x] **"Does `PEO_ALT_SEEDS` want to be larger than 4?"** Yes, 8. Sixteen adds exactly
      zero and thirty-two buys 0.20 bips for +49 ms on the corpus maximum. The pool is
      monotone in the constant and shares the UNCHANGED 4M ledger, so the work envelope
      does not move ([0085](experiments/0085-stage6-move-sampling-metered-by-fill.md)).
- [ ] **An `(n, nnz)` gate is not an absolute-constant work bound whenever the cost is
      driven by the FACTOR.** A `SmallScore` pass costs `words * (n + fill)` and `fill`
      is bounded only by `n(n+1)/2`; dropping stage 6's `nnz <= 3000` half took
      `graphpart_clique-70` to 1.771 s against the 2 s cap while scoring +0.18 bips.
      Audit every other gated path the same way: is the quantity that sets its cost an
      input the gate can bound, or an output the input chooses?
- [ ] **`0084`'s seed-source widening, tested as diversity rather than as quantity.**
      The stages that assign `best_perm` directly are still not represented in the
      alternate-seed pool. But 8x more seeds from the existing source bought 0.20 bips,
      so the pool is not starved for entries - what would pay is seeds from a different
      LINEAGE. Keep the shipped seeds first so the result stays monotone.
- [x] **"Re-fit the dev-to-hidden translation."** Done, and the answer is that it is
      **mechanism-shaped**, not family- or bucket-shaped. `0084` INTRODUCED the alternate-seed
      chains: 4.11 dev bips -> 2.61 absolute hidden bips, 0.64. `f8941f7f` DEEPENED the same
      mechanism's pool from 4 seeds to 8 under the identical ledger, one hour later: 0.61 dev
      bips -> **0.02**, i.e. **0.03**. Introducing fires wherever a precondition holds and
      transfers; deepening fires only where the shallow version ran out, which is rare and
      corpus-specific. Promotion floor: **~1.35 dev bips for an introduction, ~26 for a
      deepening** ([0085](experiments/0085-stage6-move-sampling-metered-by-fill.md)).
- [ ] **Re-price every open queue item as "introduction" or "deepening" before building it.**
      A constant sweep, a budget increase, a pool depth, an extra seed set and a wider gate are
      all deepenings, and no deepening left in this tree has 26 dev bips of reach. The queue is
      mostly deepenings.
- [ ] **Gate an additive change on the DISTRIBUTION of per-row time increases.** `6ce0721`
      moved the dev corpus maximum by +14.9 ms against a +50 ms tolerance (21.7 sigma), passed
      everything, and `failed`. It also raised **104 of 300 rows by more than 25 ms with a
      153.7 ms maximum**. Every gate anyone here uses is a statistic of the dev corpus's own
      tail; none is a statistic of the change. Ask instead: how much does this add to one row
      inside its gate, what fraction of rows are inside, and would a row at 90 % of the cap
      survive it?

- [x] **"What is left at `n <= 300` that the move class cannot reach?"** Answered by
      [0086](experiments/0086-cross-candidate-subtree-transplant.md), and the answer generalised
      far past that band: donate the orderings `consider` retains into the incumbent's
      elimination-tree blocks. 13.73 dev bips wired terminally, ~35.6 unmetered. Only 0.51 of
      those bips land in `lt_1k`, so whatever holds that bucket at 0.8897 is still unidentified.
- [ ] **Raise `TRANSPLANT_LEDGER` from 600k to 1M units.** +11.6 dev bips (22.91 vs 11.29) for
      12.7 -> 21.1 ms local, 17.3 -> 28.8 ms on the slowest reported host, and 11 rows crossing
      +25 ms. Inside the only per-row increase distribution ever measured cap-safe here
      (61 rows > 25 ms / 26 > 50 ms / 138.8 ms worst, `f8941f7f`, `rejected` so it ran to
      completion) but outside a cautious default. Decide it on `0086`'s hidden verdict, and treat
      a `failed` at 600k as retiring the whole axis rather than as an argument for 400k.
- [ ] **Transplant the same donors MID-pipeline, before the terminal PEO chain.** `0085` measured
      a factor of three between the post-hoc and shipped value of a mid-pipeline insertion, so
      the reach could be ~40 bips - but it forfeits `0086`'s by-position discharge of the
      realized-chain question and needs the full envelope re-solved.
- [ ] **Iterate the transplant.** One pass only ships. Re-postordering the accepted result and
      transplanting again is untested and doubles the ledger.
- [ ] **Every small-graph gate is evaluated on the RAW `(n, nnz)`, never on the K=3 residual
      core.** `mod.rs` tests `n >= 12 && n <= 300 && pattern.nnz() <= 3_000` on the raw pattern,
      so a 7295-vertex row whose core is 95 vertices is refused by stage 6, by `rgreedy::search`
      and by MinFill alike. On the core the same gates admit 53 more rows to stage 6, 74 to the
      exact LNS and 34 to MinFill; `prefix_flops >= mine` kills none of the 53, and one row
      (`blend718`, 62 780 -> 61 714) exhibits an ordering the shipped tree does not find. On that
      row the win is the PORTFOLIO, not the polish: `prefix + min-degree/min-fill/MCS` already
      beats shipped, which says production's own core portfolio is beatable by three textbook
      heuristics.
- [ ] **A python surrogate of a mechanism whose inputs come from the production tree is not even
      a lower bound.** The transplant measured 1.0 dev bip and zero rows better than shipped
      against a six-candidate python donor portfolio, and ~35.6 bips against the orderings
      production actually retains. Promote anything that consumes a pipeline artefact straight to
      an in-binary `#[cfg(test)]` probe.
