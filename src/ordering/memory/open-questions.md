# Open questions

The research queue: leads worth chasing, gaps in the knowledge base, and
hypotheses not yet tested. Add a line whenever you notice one; resolve it by
linking to the page (experiment, technique, or literature note) that answers
it, rather than deleting it — a resolved question is a useful signpost.

## Active

- [ ] **Structured relabelings, not random ones (top lead).** The shipped
      multi-start uses uniform-random `Q` (Fisher-Yates), which is the dumbest
      possible choice — it works only because AMD's tie-breaking is numbering-
      sensitive. A *structured* `Q` should do better at identical cost: seed the
      relabeling from RCM, from a partitioner's block order, or from a previous
      restart's winning permutation (a hill-climb rather than i.i.d. sampling).
      The budget machinery in [0003](experiments/0003-relabelled-amd-multistart.md)
      already prices candidates, so this is cheap to test — reuse
      `probe_relabel_budget` and swap `relabel`.
- [ ] **Does the budget want to be non-uniform across buckets?** The shipped
      `RELABEL_BUDGET` spends the same ~0.3 s everywhere, but `gt_10k` carries
      weight 0.40 over only 45 matrices (~4.4× the per-matrix leverage of
      `lt_1k`). A bucket-weighted budget — more restarts where a win is worth
      more — was never tested. Note `n` is known inside `order()`, so this stays
      a pure function of `(n, nnz)`.
- [ ] **The big tied matrices are gated out of everything.** `faclay75`
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
- [ ] Do any ML/RL-guided ordering ideas fit a stdlib-only, deterministic,
      2 s/matrix `order()`? Survey the literature before assuming yes/no.
- [ ] The hand-rolled `nd_order` / `ndfm_order` use a plain **degree sort** at
      their leaves (`ND_LEAF=200`, `NDFM_LEAF=100`) and for unsplittable
      separators. The textbook hybrid hands leaves to minimum degree instead.
      Cheap to try (AMD on the induced subgraph) — but note their gate is nearly
      a subset of the METIS gate, so the upside may be small.

## Resolved

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
