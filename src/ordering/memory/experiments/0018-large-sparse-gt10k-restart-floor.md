# 0018 — Large-sparse gt_10k restart floor (gabriel10-class)

- **Date:** 2026-09-02
- **Score:** 0.860780 → **0.860589** (geomean flop vs AMD; fill 0.955916 → **0.955822**)
- **Status:** win (local, pending eval)
- **Parent:** promoted `216fd62` / submission `7c25e77` (eval 0.881683)

## Hypothesis

Open-questions.md already named the stuck `gt_10k` ties: `gabriel10` (n=244056),
`unitcommit_200_100_1_mod_8` (n=146830), `faclay75` (n=272878). The budgeted
relabel allocator computes `base_r = (budget / nnz).min(cap)` with
`budget = 500_000` for `n >= 10k`. At nnz ≈ 1.15M that is **zero restarts**,
so those matrices receive only the AMD floor and sit at ratio 1.000.

Experiment 0016 already unstarved the 250k–350k nnz band (`base_r.max(8)` when
`nnz <= 5n`). The next band — nnz 350k–1.2M, still sparse (`nnz <= 6n`), still
non-hub (`max_deg * 50 <= n`) — is the same mechanism one density step up.

## What changed

In `relabel_restarts_tuned`, after the 0016 sparse-network floor:

```
} else if nnz > 350_000 && nnz <= 1_200_000 && n <= 250_000
    && nnz <= 6 * n && max_deg * 50 <= n && n >= 10_000
{
    base_r.max(4)
}
```

Safety constraints, all required:

1. `nnz <= 6 * n` — planar/mesh/network density only. Excludes dense NLP
   KKT blocks (gams05-class, nnz/n ≈ 14).
2. `max_deg * 50 <= n` — the same hub discriminator that protects
   `ringpack_30_2`. Extra AMD on a hub graph is a 2.0 s timeout.
3. `n <= 250_000` — **measured**, not guessed. A first draft without this
   cap timed out `faclay75` (n=272878, nnz=1.38M, nnz/n≈5) on the local
   corpus (`results.tsv` FAIL row 1788386477). The n cap keeps gabriel10
   (244k) in and faclay75 out.
4. Floor of **4**, not 8. Four extra AMD passes on a 244k/852k synthetic
   cost ~0.12 s locally. Eight would be closer to the 2.0 s envelope once
   the grader's ~1.6× timing noise is included.

Best-of floor is unchanged: extra candidates can only lower flops.

## Result

Full 300-matrix local run after the n-cap (row 1788386652):

| Bucket | Count | Weight | Before (0.860780) | After (0.860589) |
|--------|------:|-------:|------------------:|-----------------:|
| lt_1k  | 147 | 0.30 | 0.8965 | 0.896482 |
| 1k_10k | 108 | 0.30 | 0.8903 | 0.890302 |
| gt_10k |  45 | 0.40 | 0.8119 | **0.811384** |
| overall | 300 | 1.00 | 0.860780 | **0.860589** (−1.91 bips) |

`faclay75` stays at 1.000 and **returns inside the cap**. The FAIL without
the n-cap is the existence proof that the bound is load-bearing.

## Why it won

Same lottery as 0003/0016: AMD's output is a function of the vertex numbering,
so `AMD(Q A Qᵀ)` composed back through Q is a different minimum-degree
ordering. The 350k–1.2M sparse giants were getting **zero tickets**. Four
tickets is enough to break a subset of the 1.000 ties without paying the
faclay75 tax.

lt_1k / 1k_10k are bit-identical (the new branch is gated `n >= 10_000`),
so the whole delta is `gt_10k`. That is the highest-weight bucket.

## Follow-ups

- Do **not** raise the n cap past 250k without a per-matrix timing probe on
  faclay75-class shapes. Local evidence says 4 extra AMD passes there die.
- `acopf_case9241pegase_qcqp` (n=313k) is still gated out; it is denser and
  larger than faclay75 and is not a candidate for this floor.
- Next leverage after this: relabel RCM/Sloan/ND (open-questions.md top
  lead) or a 1k_10k exact-search extension. Do not add unbounded candidate
  families — hidden-eval mid-band matrices are slower than any dev matrix
  (submissions 5a05758 / 14cbc219 / 952cbbf / f66d60b all SIGKILL'd at 2.0 s).

## Links

- Predecessor floor: [0016](0016-sparse-gt10k-restart-floor.md)
- Relabel lottery: [0003](0003-relabelled-amd-multistart.md)
- Hub discriminator: [0011](0011-hub-gate-and-floors.md)
