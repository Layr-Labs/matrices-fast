# 0290 — the simplicial reduction shrinks cores but moves nothing

- **Date:** 2026-09-15 (iter82)
- **Base:** iter81 `b21879f` (dev probe **0.790253862267**, 300/300 `COUNTS`).
- **Change (measured, reverted):** add `core_lift::reduce_simplicial` — the
  degree-bounded reduction **plus the simplicial rule at any degree** — as a
  separate core attempt behind the test seam `SSI_REDUCE_SIMPLICIAL`, and widen
  `order_core`'s small-core quality block from `1000 <= n < 10_000` to a
  `SSI_CORE_QUALITY_MAX_N` seam.
- **Result:** **NEGATIVE — 300/300 flop records bit-identical, SCORE 0.790254
  both ways.** The rule fires on most rows and shrinks cores by 20–98 %, and no
  output changes anywhere. Reverted; the patch is kept at
  `.session-backup/iter82-simplicial-negative.patch` and recoverable from
  `git stash list` (`iter82 simplicial negative result`).
- **Status:** sealed. Not shipped.

## Why it was tried

The public board's only window entry that beat the frontier score is
`1371aa9` / PR #738 (0.840489, rejected for the 1 bip bar): a clean-room port of
the promoted base plus a **kernel-core exact search** — "extends the degree-≤3
exact-elimination prefix with fill-bounded *simplicial* and *almost-simplicial*
rules to a fixpoint under a 40 M pair-check ledger, then runs the existing
bounded exact machinery on the 24–86 % smaller kernel core". Our `core_lift`
reduces by live **degree** (`max_row_deg` ∈ {3,5,4,2,6}), which is a different
rule: it eliminates a vertex whatever its neighbourhood looks like and pays the
closure fill. A vertex whose live neighbourhood is *already* a clique needs no
fill at all, so it is a legal exact elimination step at **any** degree — and it
is exactly the vertex a degree cap cannot reach. That is a genuinely new rule
for this tree, and it is monotone by construction: it only ever removes a vertex
that the closure loop would have added no edge for.

## What was implemented

`core_lift::reduce_impl` gained a second const parameter, `SIMPLICIAL`. With it
set, the walk admits **every** live vertex at its current degree; a popped
vertex with live degree above `max_row_deg` is eliminated only when

```
  for all pairs (a, b) in N_live(v):  key(a, b) ∈ edges
```

holds. Verification is charged *before* it is attempted — `d(d-1)/2`
edge-membership tests against `SSI_REDUCE_SIMPLICIAL_PAIR_BUDGET` (1 M) — and
the heap is min-degree, so once the cheapest remaining candidate is
unaffordable the walk stops. A verified elimination skips the closure loop
entirely, since the closure inserts nothing.

The `SIMPLICIAL = false` instantiation reproduces the old walk exactly (the
existing `reduce` / `reduce_checked` call sites are unchanged, and the
reference cross-check tests still pass).

The rule is an exact elimination step, and the tests prove it the only way that
matters: `simplicial_rule_is_an_exact_elimination_step` checks the **split
identity** on every graph with `n <= 5` and the identity/reversed core
permutations,

```
  flops(pattern, splice(lift, core_perm))
      == lift.prefix_flops + flops(core_graph, core_perm)
```

which fails if a simplicial elimination forgets an edge, plus
`assert_symmetric_core` and `prefix.len() + core_n() == n`.
`simplicial_rule_peels_a_high_degree_clique` pins the mechanism on K8 + a
5-cycle at a degree cap of one: the bounded rule eliminates **0** vertices
(core 13), the simplicial rule eliminates 8 (core 5, the 5-cycle). Release
`core_lift` suite: **10 passed / 0 failed**.

## What it bought

Core sizes with the rule on, at `max_row_deg = 3`, over the small/dense bands
(`SSI_REDUCE_TRACE=1`):

| row | n | nnz | core n | core nnz | prefix |
|---|---:|---:|---:|---:|---:|
| `squfl030-150` | 13 680 | 36 000 | **180** | 9 000 | 13 500 |
| `emfl100_5_5` | 21 925 | 52 300 | **650** | 5 450 | 21 275 |
| `emfl050_5_5` | 13 175 | 32 300 | **650** | 5 450 | 12 525 |
| `mpbp_34` | 11 556 | 40 860 | 2 610 | 19 788 | 8 946 |
| `chimera_selby-c16-02` | 2 031 | 10 878 | 2 000 | 10 868 | 31 |
| `supplychainr1_053050` | 16 640 | 57 800 | 3 430 | 31 440 | 13 210 |
| `nuclear104` | 39 098 | 257 806 | 14 450 | 222 654 | 24 648 |
| `gams05` | 17 364 | 252 910 | 9 075 | 234 790 | 8 289 |
| `kissing2` | 20 772 | 401 024 | 10 767 | 381 010 | 10 005 |
| `pooling_sppc1pq` | 14 100 | 477 680 | 7 600 | 464 680 | 6 500 |

and the score does not move. Full 300-row arm
(`SSI_MARK_NOSCORE=1 SSI_REDUCE_SIMPLICIAL=1`, one binary/one session):

| arm | SCORE | lt_1k | 1k_10k | gt_10k | `COUNTS` |
|---|---:|---:|---:|---:|---:|
| baseline `b21879f` | **0.790254** | 0.8873 | 0.8373 | 0.6822 | 300 |
| + simplicial rule | **0.790254** | 0.8873 | 0.8373 | 0.6822 | 300 |

`diff` of the `(row, candidate flops)` pairs: **0 differing lines**. Not one
row moved. Adding the widened `order_core` quality gate on top
(`SSI_CORE_QUALITY_MAX_N=60000`, which lets the relabelled-AMF tie-breaks and
the exact min-fill run on the now-tiny cores of `n > 10 000` rows) is also
identical on every sampled row — including the five `gt_10k` anchor ties, where
the core is 180–650 vertices and the machinery still lands exactly on AMD's
value.

## What this means

1. **A smaller core is not by itself a better ordering.** The core search's
   output is a *whole-graph* value, `prefix_flops + core_flops`, and the
   pipeline already reaches the same value on the original, larger core. Every
   core it can now build was already inside the reach of some other stage:
   the terminal exchange, the nine sparse spans, the PEO chain, or the
   relabelled lotteries.
2. **The five `gt_10k` anchor ties are not a reduction problem.** They are the
   highest-leverage rows on the corpus (`gt_10k` weight 0.40 over 45 rows;
   one 10 % win is ≈9.4 bips). `probe_tie_forensics` shows all five carry fill
   (`fillfree = 0`, fill 9 225 … 260 722), so a tie is *not* vacuous — but
   `probe_tie_headroom` finds **21 diverse candidates** (six AMD option pairs,
   three AMF α, METIS/tuned/high-trial, Scotch, RCM, Sloan, ND, six custom
   metrics) all landing on **exactly 1.00000000**, the same value AMD returns.
   Combined with the 180-vertex core above, the reading is that these five are
   at the optimum the whole family converges to, not at a search failure. A
   `fillfree = 0` tie is evidence about chordality, not about headroom.
3. **The core-size axis is therefore closed as a value lever** on this tree.
   What remains open is core *stiffness*: every mechanism in the tree is a
   min-degree-flavoured or window-local search, and the simplicial lift proves
   the graphs can be taken apart exactly. A future session that wants this
   substrate should bring a genuinely new objective on the core, not a bigger
   budget for the existing ones.

## Verification

- `cargo test --release -p ssi-candidate-worker --offline --locked core_lift::`
  — **10 passed / 0 failed** (two new tests).
- Full 300-row dev probe, one binary, `SSI_MARK_NOSCORE=1`: baseline
  `iter82-base-full.log` **0.790254**, simplicial `iter82-simp-full.log`
  **0.790254**, 300/300 `COUNTS`, zero differing flop records.
- `probe_eval_audit` on the base: `rows=300 leak_rows=0 scored_candidates=3121`,
  `recoverable_bips = 0.00` — the stale-`best_flops` family still has no dev
  headroom, so the null is not a bookkeeping artifact.
- `probe_tie_forensics` (full corpus): `lt_1k` 14 fill-free / 133 with fill,
  `1k_10k` 2 / 106, `gt_10k` 0 / 45.
- Reverted: `git stash list` → `iter82 simplicial negative result`; patch at
  `.session-backup/iter82-simplicial-negative.patch`.

## Links

- [0289 restore the promoted `Game` buffer lifecycle](0289-restore-promoted-buffer-lifecycle.md)
- [0281 the terminal class gate as a resource law](0281-resource-law-class-gate.md)
- [0091 residual-core exact minimum fill](0091-residual-core-exact-minimum-fill.md)
- [0062 reduce-then-AMF terminal](0062-reduce-then-amf-terminal.md)
