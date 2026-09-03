# 0041 — Medium-only subtree block cap

**Date:** 2026-09-03

**Frontier:** submission `26932eb`, commit `344a5d2`, hidden **0.874601**.
Its public dev result is **0.849487** with fill **0.947766**.

**Result:** dev **0.849194** (−0.000293), fill **0.947627**.

**Status:** full 300-matrix trusted run passes; submitted for private validation.

## One-line functional change against the promoted frontier

The promoted source already gives `n < 1000` a fixed-work small-tree config:
`min_s=16`, `max_s=256`, `max_blocks=16`, and `budget=2M`. This experiment
leaves that block unchanged and adds one size-bucket-specific choice:

```rust
} else if n < 10_000 {
    cfg.max_s = 256;
}
```

For `1000 <= n < 10000`, the first two elimination-tree refinement rounds now
search subtrees of at most 256 vertices instead of 384. Block count, stream
count, operation budget, ranking, minimum size, and every other candidate remain
unchanged. For `n >= 10000`, the accepted `max_s=384` path remains unchanged.

## Hypothesis

The public sweep behind submission `26932eb` showed that the earlier small-tree
cap of 512 was too broad: a large eligible subtree could absorb most of the
useful structure while the bounded exact search spent its allocation inside a
single basin. A plateau around 224–288 generalized across the two public-corpus
halves, with 256 chosen as the center.

The natural follow-up was whether the inherited base cap of 384 was also too
large outside `lt_1k`. The mechanism should plausibly extend to medium trees,
but there was no reason to assume one cap would fit every size. The experiment
therefore began with a deliberately falsifiable global point, then used the
three score buckets as controls.

## Experiment A — global 384 to 256 (negative overall)

First, `SUBTREE_CFG.max_s` was changed globally from 384 to 256. The full trusted
run completed, but the aggregate score worsened:

| configuration | lt_1k | 1k_10k | gt_10k | score |
|---|---:|---:|---:|---:|
| prior candidate in that run series | 0.8933 | 0.8752 | 0.7969 | 0.849309 |
| global base `max_s=256` | 0.8933 | **0.8742** | **0.7991** | **0.849878** |

This is a useful negative result, not a failed mechanism. The medium bucket
improved by about 0.001, while the high-weight large bucket regressed by about
0.0022. One global cap combines two effects with opposite signs, and the 0.40
weight on `gt_10k` makes the large regression dominate.

Per-matrix output was consistent with that split. Several medium instances
found cheaper refinements, while some large elimination trees lost productive
384-node blocks. This is exactly why a size-specific heuristic is preferable to
claiming that the small-tree optimum is universal.

The global edit was reverted immediately. No global 256 cap is present in the
candidate.

## Experiment B — isolate 256 to the medium bucket

`subtree_cfg_for(n)` already owns the size-dependent configuration, so the
follow-up puts `max_s=256` only on `1000 <= n < 10000`. This preserves the
accepted small allocation below 1000 and restores `max_s=384` at and above
10000.

The exact bucket result is:

| bucket | frontier | candidate | delta |
|---|---:|---:|---:|
| `lt_1k` (147, weight 0.30) | 0.893893 | **0.893893** | 0 |
| `1k_10k` (108, weight 0.30) | 0.875176 | **0.874198** | −0.000978 |
| `gt_10k` (45, weight 0.40) | 0.796916 | **0.796916** | 0 |
| composite | 0.849487 | **0.849194** | −0.000293 |

The aggregate arithmetic cross-check is exact to the reported precision:

```text
0.849487 + 0.30 × (0.874198 - 0.875176) = 0.8491936
```

This matches the trusted scorer's rounded **0.849194**. The unchanged small and
large bucket values are the key attribution control: the branch condition says
the edit cannot reach them, and the measured scores confirm that it does not.

## Why the medium cap is structurally different from overfitting

The selector uses only the matrix dimension. It does not inspect a matrix name,
hash, values, right-hand side, environment variable, filesystem path, timing,
or corpus position. Every matrix in the medium size range receives the same cap.
The threshold is the challenge's own scoring boundary, not a dev-instance
fingerprint, and the underlying mechanism—avoid letting one broad subtree
consume a bounded local-search allocation—does not depend on corpus identity.

The candidate still returns a deterministic permutation. `subtree_refine`
ranks eligible blocks from the incumbent column-count contribution, searches
disjoint blocks with fixed seeds and fixed operation budgets, and applies
results in deterministic position order. Every completed ordering is validated
and re-scored on exact predicted factorization flops before acceptance.

There is also an explicit negative control: applying the same setting to large
trees worsened their bucket and was rejected. The submitted gate is based on
that mechanistic split, not on a table of special cases.

## Timing and hidden-safety argument

This revision adds no candidate, pass, stream, block, or operation. The only
change lowers the maximum eligible subtree size from 384 to 256 in two existing
medium rounds. The maximum block count remains 32, the per-block budget remains
1M, and the stream count remains one. Requested work is therefore no greater
than the promoted frontier's work.

The smaller cap also reduces the size of the local induced adjacency bitsets and
the maximum cost of graph construction and elimination within any selected
block. Some different blocks may become eligible, so no wall-clock claim is made
without a direct timing table, but the deterministic operation ceilings do not
increase and the maximum local graph dimension decreases.

This safety property matters because experiment 0040's additive terminal
whole-graph search passed public validation and then failed private validation.
The CLI did not publish the hidden failure category, but additive work was the
only new resource risk. All of that terminal code, including the `rgreedy.rs`
seed-salt change, was removed before this experiment. The source in 0041 is the
promoted algorithm plus fixed-work subtree reallocation only.

The accepted small-tree change from `26932eb` is also retained exactly. Its
requested work is the same 32M ceiling as the preceding `8 × 4M` version and it
already passed private validation. This submission asks the grader to validate
only the medium cap's effect against that frontier.

## Verification

The final source was run with:

```sh
yukon run
```

The command performed the network-denied sandboxed candidate build, purity and
license checks, then ran all 300 development matrices. `order()` was invoked
twice for every matrix under the hard two-second worker watchdog. The run had:

- no timeout;
- no panic or abnormal worker exit;
- no invalid or non-bijective permutation;
- no determinism mismatch;
- no memory-cap failure;
- no purity or dependency-policy failure.

Final `score.json`:

```text
score  0.849194
fill   0.947627
lt_1k  0.893893 / fill 0.962420
1k_10k 0.874198 / fill 0.959263
gt_10k 0.796916 / fill 0.927806
```

No dependency or lockfile changed. The functional edit is confined to
`src/ordering/mod.rs` and uses the Rust standard library plus the already
reviewed ordering implementation inherited from the frontier.

The trusted parent test suite had also passed earlier in this session: 52
non-ignored unit tests, 3 exact-equivalence tests, 2 scorer cross-checks, 4
security-boundary tests, and 5 time-cap tests; 2 explicitly ignored platform
probes were not run. The final functional change does not touch the harness or
those test paths, and the subsequent full `yukon run` is the authoritative
candidate check.

`cargo fmt --all --check` is not green on the inherited repository: it reports
extensive pre-existing formatting differences across trusted and candidate
files. No formatter was run, because that would create a broad unrelated diff.
`git diff --check -- src/ordering` is clean.

## Result and next question

The medium-only cap is a clean fixed-work improvement: **0.849487 → 0.849194**
with both non-target buckets unchanged. It should be at least as safe as the
promoted frontier because it adds no computation and reduces the maximum local
subproblem size.

The experiment also shows that `max_s` is genuinely bucket-dependent. Future
sweeps should keep `gt_10k` isolated and examine values near 384 rather than
copying the medium optimum downward. For the medium tier, 256 is one supported
point, not a proven optimum; the next honest sweep is a small plateau check at
224, 288, and 320 with half-corpus and drop-top-mover robustness columns.

