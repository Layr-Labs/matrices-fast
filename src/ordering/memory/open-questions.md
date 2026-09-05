# Open questions

The research queue: leads worth chasing, gaps in the knowledge base, and
hypotheses not yet tested. Add a line whenever you notice one; resolve it by
linking to the page (experiment, technique, or literature note) that answers
it, rather than deleting it — a resolved question is a useful signpost.

## Active

- [x] **RESOLVED (partly negative) — Should search budget be scaled by the AMD
      anchor margin `best_flops / amd_flops`?** Answered by
      [0060](experiments/0060-ledger-gated-terminal-escalation.md): margin
      *describes* where gain lives (conversion 41.7% below margin 0.50 versus
      4.8% at ties; mean gain among movers 6.5e-3 versus 1.0e-3 in the middle
      bands) but is the **wrong variable to gate on**, because the binding
      constraint is time and margin does not predict time. The three matrices
      that break the cap first are all deep below the anchor, so margin-scaled
      escalation spends most exactly where it can least afford to. Every
      margin-keyed depth policy tested scored strictly worse than a uniform
      depth under the same cost gate. **The variable that works is a requested-
      work ledger.**
- [x] **RESOLVED (negative) — Can `(n, nnz)` gate an expensive terminal stage?**
      No. 216 `(n, nnz)`-keyed gates crossed with margin-scaled depths produced
      zero configurations under the timing bar
      ([0060](experiments/0060-ledger-gated-terminal-escalation.md)). The chain
      rounds are conditional on one another, so size does not predict pipeline
      cost: `crudeoil_lee1_07` (n=3670) costs 1.14 s while `batchs121208m` costs
      a fifth of that. Use `work_spent` instead — it is a pure function of the
      pattern and its upper envelope is clean (every matrix below 2.0e9 finishes
      in ≤ 0.713 s).
- [x] **RESOLVED (against the received wisdom) — Are matrices tied at 1.0000
      dead?** Not entirely. The prior note's claim that no search converts them
      is correct for the elimination-game LNS and **wrong for subtree
      refinement**: 3 of 83 eligible ties move, and because all three are
      `gt_10k` they are worth ~5.3 bip
      ([0060](experiments/0060-ledger-gated-terminal-escalation.md);
      `gasprod_sarawak81` 5.53%, `popdynm200` 3.32%). Only 3.6% convert, so ties
      remain a bad *target* — but excluding them from a monotone pass costs
      3.0–4.3 bip. Do not aim at ties; do not exclude them either.
- [x] **RESOLVED (negative) — Buy extra search trajectories with `streams`?** No.
      Setup is not the cost (median 0.0003 s, max 0.052 s); the search is, so
      streams cost linearly. At ~4× cost, 4 streams return 2.14× the log-gain of
      1 stream while 4 *rounds* return 3.07×, because re-postordering gives a new
      block decomposition and extra streams only re-roll the same one
      ([0060](experiments/0060-ledger-gated-terminal-escalation.md)).
- [ ] **Give the ledger-excluded matrices a SMALLER escalation.** The 75 matrices
      above the 2.0e9 ceiling hold roughly half the remaining time-feasible gain
      (the oracle bound is −19.11 bip; 0060 ships −7.47). The way in is not a
      higher ceiling — that admits the 1.38 s matrices — but a cheaper round for
      them, e.g. 8M × 8 blocks instead of 32M × 32. The ledger already identifies
      exactly which matrices those are.
- [ ] **Make the ledger count work ISSUED, not requested.** `subtree_refine`
      knows how many word-ops it actually consumed. Returning that would turn the
      ledger from a safe upper envelope into a cost estimator
      (`corr(log10 ledger, secs)` is only 0.399 today) and would let the ceiling
      sit much closer to the cap.
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
      zero wins, with 0.071 s worst combined added local time. A multi-seed test
      remains open, but one-pass production additions are not supported.
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
- [~] **PARTLY ANSWERED — How much of the remaining headroom is even measurable
      on 300 matrices?** [0060](experiments/0060-ledger-gated-terminal-escalation.md)
      ran the bootstrap this question asks for (2000 resamples with replacement,
      re-aggregated) on a −7.47 bip change: mean −9.76 bip, 95% interval
      **[−20.92, −2.10]**. So a 7 bip dev win is worth "somewhere between 2 and
      20 bip" on a fresh corpus — the point estimate is nearly meaningless and
      only the sign is safe, and only because the mechanism is monotone. Still
      open: fold the bootstrap into a probe rather than a one-off script over
      `probe_timing_and_score` output, and re-run it over the past promotions to
      see which were real.
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

## Resolved by 0061 (conjunctive cost gate) — and by the 0060 hidden failure

- [x] **Is a requested-work ledger enough to gate an expensive terminal stage?**
      **No, and this cost a submission.** [0060](experiments/0060-ledger-gated-terminal-escalation.md)
      shipped a ledger-only gate and **failed** on the hidden corpus. The ledger
      counts only the exact-search stages and is blind to the O(nnz) candidate
      construction, so it calls `ringpack_30_2` free (ledger 48M, one fortieth of
      the ceiling) when it actually costs 0.714 s. On dev the low-ledger matrices
      happened to top out at 0.714 s; nothing in the mechanism made that true.
      A cost gate needs a bound on **every** cost axis, conjunctively — see
      [0061](experiments/0061-conjunctive-cost-gate.md), where `ledger < 5e8 AND
      nnz <= 150k` excludes all 31 dev matrices over 0.9 s.
- [x] **Does the `budget` knob in `subtree_refine` actually bind, or do blocks
      terminate early?** **It binds, almost perfectly.** New probe
      `probe_budget_saturation`: a 4x budget rise gives a median **3.976x** in
      wall time, and corpus-total time doubles with every budget doubling from
      4M to 128M. Budget is therefore a direct time knob and requested work is a
      *tight* bound, not a loose one — which is also why 0060's 4.1e9-op request
      against a 2.0e9 ceiling was a real ~0.5-0.7 s commitment.
- [x] **Is it safe to size an escalation against the ceiling that admitted the
      matrix?** No. Keep the admitted work small in **absolute** terms. 0060
      requested more than 2x its own ceiling; 0061 requests 1.79e9 against a
      5e8 ceiling but caps the measured worst added cost at 0.229 s.
- [x] **Do ties convert?** Reconfirmed at the tighter gate: 2 ties move, both
      `gt_10k`, and `gasprod_sarawak81` (1.0000 → 0.9604) alone is ~2.6 of the
      6.22 bip. Ties are a bad *target* but must not be *excluded*.

- [ ] **Can the excluded matrices be reached with a cheaper round?** The oracle
      bound is −19.11 bip and 0061 captures 6.22 of it; the rest sits on the 116
      matrices the gate excludes. Try 4M x 8 blocks for them — bounded on both
      axes by construction — rather than loosening either gate.
- [ ] **Make the ledger count work ISSUED rather than requested.** That turns it
      from an envelope into an estimator and is the only thing that would justify
      a ceiling anywhere near the cap. Requires threading a counter out of
      `subtree_refine` and `search`.
