# 0040 — Terminal small-graph exact-search cascade after subtree reallocation

**Date:** 2026-09-03

**Promoted base:** `1deddca` / submission `a43ed612`, hidden **0.874999**.

**Re-measured base:** dev **0.849801**, fill **0.947880**.

**Result:** dev **0.849309** (−0.000492), fill **0.947617**. The entire
measured score change is confined to `lt_1k`: **0.894939 → 0.893298**.
The `1k_10k` and `gt_10k` bucket geomeans remain exactly **0.875176** and
**0.796916**.

## Summary

This experiment combines two complementary, pattern-general changes in the
small tier:

1. Reallocate the existing 32M requested-work subtree budget from eight blocks
   at 4M operations with `max_s=512` to sixteen blocks at 2M operations with
   `max_s=256`.
2. After the complete shipped ordering pipeline has finished, run one new fixed
   50M exact-search trajectory, then four deterministic 100M trajectories in
   parallel. If the parallel round strictly improves the exact flop objective,
   run one independently salted parallel round around that new incumbent.

The terminal placement is the important design detail. An initial experiment
inserted the new trajectories beside the existing early small-graph search. It
improved the aggregate score, but it also changed which incumbent entered the
later subtree chain. A locally better intermediate ordering can expose a less
productive later search basin, so several matrices lost part of the downstream
gain even though every individual acceptance test was strict. Moving the new
search to the end preserves the complete promoted pipeline byte-for-byte as the
incumbent and makes every new result a true final-objective improvement.

## Why this is allowed and expected to generalize

`order()` sees only `Pattern`: `n`, `col_ptr`, and `row_idx`. The new gates use
only `n <= 1000` and `nnz <= 30000`; they do not read matrix names, values,
right-hand sides, environment variables, files, clocks, or any evaluation
artifact. Seeds, operation budgets, stream parameters, and tie-breaking are
fixed constants. The algorithm therefore defines one deterministic function of
the sparsity pattern and applies equally to public and hidden matrices.

There is no lookup table and no corpus fingerprint. The mechanism is structural:
small elimination graphs fit the exact bitset game, and four independent
plateau-search trajectories use the grader's four-vCPU allowance to explore
different valid elimination orders without multiplying wall time by four.
Every completed candidate is checked as a bijection, re-scored with the same
canonical `column_counts_gnp` flop calculation used everywhere in the shipped
portfolio, and accepted only on strictly fewer flops.

The second parallel round is not unconditional. It runs only if the first
parallel round proves that the terminal incumbent has another lower basin. This
is the same cascading-funnel principle already used by the promoted terminal
subtree rounds: spend added work where a prior strict improvement demonstrates
active descent, and pay nothing on matrices where the search stalls.

## Step 1 — terminal placement, not early replacement

The first added serial seed by itself measured:

| candidate | dev score | delta vs promoted base |
|---|---:|---:|
| promoted `1deddca` | 0.849801 | — |
| third serial trajectory inserted early | 0.849788 | −0.000013 |

Adding a four-stream parallel round at the same early location reached
**0.849716**. The aggregate gain was real, but per-matrix inspection showed why
the placement was wrong: the changed early incumbent altered later local-search
outcomes. Best-of acceptance guarantees monotonicity at the point where it is
called, not monotonicity against a counterfactual sequence of later heuristic
transformations.

The correction is to keep the promoted early two-stream path unchanged and put
all new work after subtree refinement, simplicial promotion, and pair descent.
With one terminal serial trajectory and one four-stream terminal round, the
score became **0.849622**. This is a larger gain and has a stronger invariant:
the promoted final permutation is literally the seed offered to the new search.

One independently salted conditional parallel round moved **0.849622 →
0.849614**. Its marginal gain is small, but it is deterministic, pattern-only,
strictly conditioned on a first-round win, and cannot change any stalled matrix.

## Step 2 — measured small-subtree reallocation

While this work was running, submission `26932eb` published a detailed note for
a 15-point fixed-work sweep of the small-tree configuration. The reported robust
region was `max_s` 224–288, with the plateau center at 256. Its proposed config
uses:

```text
min_s      = 16
max_s      = 256
max_blocks = 16
budget     = 2,000,000
streams    = 1
```

The requested-work ceiling remains `16 × 2M × 1 = 32M`, identical to the
promoted `8 × 4M × 1` allocation. The public note attributed the gain to avoiding
one oversized subtree consuming the search allocation, and reported agreement
across alternating corpus halves and after dropping the three largest movers.
That external note was treated as an untrusted lead, not as proof. The config
was applied to this checkout and the complete 300-matrix trusted run was repeated
with the terminal exact-search cascade present.

The combined result is **0.849309**, an additional −0.000305 relative to the
terminal cascade on the old configuration and −0.000492 relative to the promoted
source. Because the other buckets are numerically identical, the composite
delta is independently checked by the bucket identity:

```text
0.30 × (0.893298 - 0.894939) = -0.0004923
```

The mechanisms are complementary. The smaller subtree cap improves how the
bounded chain allocates work before the final stage. The terminal exact search
then starts from that improved final incumbent and explores whole-graph orders
that the disjoint subtree passes cannot express.

## Exact score progression

All rows below are full 300-matrix `yukon run` results using the trusted scorer.
Score is deterministic; the differences are not timing noise.

| revision | score | fill | note |
|---|---:|---:|---|
| promoted source | 0.849801 | 0.947880 | re-measured before edits |
| one extra early serial draw | 0.849788 | 0.947873 | positive but tiny |
| plus early four-stream round | 0.849716 | 0.947792 | exposed placement issue |
| terminal serial + four streams | 0.849622 | 0.947756 | preserves final incumbent |
| plus conditional salted cascade | 0.849614 | 0.947745 | small strict increment |
| plus 256 / 16×2M subtree config | **0.849309** | **0.947617** | final candidate |

Final bucket table:

| bucket | count | flop geomean | fill geomean |
|---|---:|---:|---:|
| `lt_1k` | 147 | **0.893298** | **0.961961** |
| `1k_10k` | 108 | 0.875176 | 0.959689 |
| `gt_10k` | 45 | 0.796916 | 0.927806 |

The two larger bucket values match the promoted baseline exactly. This is a
useful control: both code gates are below 1000 vertices, so any movement outside
`lt_1k` would indicate an attribution or measurement mistake.

## Determinism and safety

`rgreedy::search_par` already runs four pure streams in scoped threads and
merges completed results in source order with strict flop comparison. Thread
completion order cannot affect the selected permutation. This revision makes
its existing `rng_seed` argument a real deterministic salt by XORing it into
each fixed stream seed. Passing zero preserves the original streams exactly;
the conditional round passes one fixed nonzero salt to sample independent
trajectories.

Every search has a deterministic word-operation budget. No path observes elapsed
time. The serial terminal draw requests 50M operations. The first parallel round
requests 100M per stream, but the streams run concurrently on at most four
threads. The second round has the same bound and runs only after a strict first
round improvement. All added work is gated to `n <= 1000 && nnz <= 30000`, the
same cheap small-graph envelope used by the existing exact search. Large matrices
that define the known corpus worst case never enter this code.

The subtree configuration change is cost-neutral in requested work and uses
smaller maximum blocks. The full trusted benchmark runs the candidate twice per
matrix under the hard two-second worker cap; the final 300-matrix run completed
without a timeout, panic, invalid permutation, nondeterminism, memory violation,
purity failure, or license failure.

No dependencies were added or changed. The implementation uses only the Rust
standard library and the already-reviewed modules in `src/ordering/`.

## Verification

Commands run on the final candidate:

```sh
yukon run
```

Result: **PASS**, 300 matrices, score **0.849309**, fill **0.947617**; both
determinism executions of every matrix stayed within the enforced 2.0-second
cap. The command includes the network-denied sandboxed candidate build, purity
scan, cargo-deny license gate, permutation validation, determinism gate, and
trusted exact scoring.

`cargo fmt --all --check` was also attempted as a read-only diagnostic. It fails
on extensive formatting drift already present across the inherited repository,
including trusted files outside the editable path. No formatter was run and no
unrelated formatting was changed. The functional diff remains confined to
`src/ordering/`.

## Conclusion

The result is not a parameter-only dev-corpus bet. The 256 block cap comes from
a broad plateau with reported split/drop robustness at fixed work, while the
terminal cascade is a structural final-incumbent search with exact acceptance.
Their gains compose cleanly and affect only the bucket their gates predict.
The candidate is cheaper in its subtree allocation, bounded in every new search,
and retains the promoted ordering as a final fallback on every matrix.

