# 0005 — Relabelled-AMF multi-start: a second lottery, not a smarter one

**Date:** 2026-09-02
**Score:** 0.876925 → **0.871827** (−0.005098). Buckets: lt_1k 0.9073→0.9063,
1k_10k **0.9069→0.8991**, gt_10k **0.8317→0.8255**. Fill tiebreak 0.962248→0.960774.
**Status:** WIN, shipped. 36 matrices better, **0 worse**, 264 byte-identical.

This is the direct consequence of [0004](0004-structured-relabelings.md)'s negative
result, and it is worth reading the two together: 0004 proved you cannot aim the
relabel lottery, so the only lever is more tickets — and the cheapest new tickets are
not more of the same draw, they are draws from a **different distribution**.

## Hypothesis

[0003](0003-relabelled-amd-multistart.md) works because AMD's tie-breaking reads the
vertex numbering: `AMD(Q A Qᵀ)` composed back through `Q` is a distinct minimum-
**degree** ordering for the price of one AMD pass, so a random `Q` per seed is a
randomized-restart MD.

**AMF (approximate minimum FILL) reads the numbering the same way.** So
`AMF(Q A Qᵀ)` composed back is a randomized-restart minimum-**fill** ordering for the
cost of one AMF pass. Plain AMF has been in this portfolio from the beginning (the α
sweep in `order()`), but it had only ever been run on the *original* numbering — the
relabel trick had never been pointed at it.

The reason to expect this to beat spending the same time on more AMD restarts is
[0004](0004-structured-relabelings.md)'s mechanism. Within one objective, the
relabeling→flops map has no exploitable local structure and the draws are effectively
i.i.d., so extra restarts buy the tail of a single distribution and saturate fast —
the budget table on `RELABEL_BUDGET` measures exactly that (past 300000, each further
0.05 s of worst case buys under 0.0002 of score). Min-fill and min-degree **disagree
about which vertex to eliminate**, so an AMF draw is not a redundant AMD draw. Two
lotteries at half the tickets each beat one lottery at full tickets whenever the
prizes are in different places.

The falsifiable prediction: the wins should land where **min-degree is already at its
own ceiling** — matrices at or near ratio 1.0000, where every degree-based candidate
converges on the anchor and only a different objective can move them.

## Implementation

One loop appended after the relabelled-AMD multi-start in `order()`, mirroring it
exactly but calling `feral_amf::amf_order_opts` with `dense_alpha: 5.0` (the α the
portfolio's base AMF candidate already uses) instead of `feral_amd::amd_order`:

```rust
if nnz <= RELABEL_AMF_MAX_NNZ {
    let amf_relabel_opts = feral_amf::AmfOptions { dense_alpha: 5.0, ..Default::default() };
    for r in 0..restarts {           // SAME restart count as the AMD loop
        let seed = r as u64 + 1;     // SAME seeds — see below
        consider(&|| {
            let q = relabel(n, seed);
            let b = permute_pattern(&scoring_pat, &q);
            /* … build CscPattern … */
            let (pb, ..) = feral_amf::amf_order_opts(&bcore, &amf_relabel_opts)?;
            Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())  // compose back
        });
    }
}
```

Three properties worth stating because they are what make it safe:

- **Score risk is structurally zero.** It is routed through `consider` like every
  other candidate, so each result is bijection-checked and kept only if *strictly*
  cheaper than the incumbent. A candidate can lower a ratio, never raise one. The
  measured `0 worse of 300` is not luck; it is the best-of floor.
- **Deterministic.** Seeds are fixed at `1..=restarts` and `restarts` is a pure
  function of nnz, so the family is a pure function of the pattern's shape. The
  harness runs `order()` twice and requires byte-identical output; re-using the AMD
  loop's seeds is deliberate, not laziness — AMF on an identical relabelled graph is a
  genuinely different candidate, so the seeds cost nothing to share.
- **Cost is bounded twice.** `RELABEL_BUDGET / nnz` already bounds the family's total
  spend the same way it bounds AMD's, and `RELABEL_AMF_MAX_NNZ = 130_000` is a second,
  independent ceiling because AMF's per-pass constant is larger than AMD's and its
  worst case is less well characterised here. Gate on **nnz, not n** — AMF's cost
  tracks nnz, so an `n` cutoff would bound the wrong quantity.

## Result

| | score | lt_1k | 1k_10k | gt_10k | worst `order()` |
|---|---|---|---|---|---|
| before (0003/0004 tree) | 0.876925 | 0.9073 | 0.9069 | 0.8317 | 0.384 s |
| **after** | **0.871827** | 0.9063 | **0.8991** | **0.8255** | 0.439 s |

**36 better / 0 worse / 264 byte-identical.** Wins in every bucket (7 lt_1k,
24 1k_10k, 5 gt_10k) and across unrelated families:

| matrix | n | nnz | ratio |
|---|---|---|---|
| `mpbp_15` | 9858 | 31692 | 0.9951 → **0.8198** |
| `risk2bpb` | 1462 | 5396 | 0.7390 → 0.6083 |
| `chp_shorttermplan2d` | 16364 | 52108 | 0.7280 → **0.5975** |
| `chimera_k64ising-02` | 1225 | 3846 | 0.7249 → 0.6237 |
| `crudeoil_pooling_dt2` | 18742 | 75910 | 0.9392 → 0.8644 |
| `chimera_rfr-02` | 2032 | 15140 | 0.7711 → 0.7000 |
| `chimera_mgw-c16-2031-01` | 2032 | 15900 | 0.9852 → 0.9257 |
| `sporttournament18` | 156 | 886 | 0.8574 → 0.7999 |
| `pooling_haverly1pq` | 31 | 96 | **1.0000** → 0.9782 |

**The prediction held.** `mpbp_15` at 0.9951 and `pooling_haverly1pq` at an exact
1.0000 are precisely the "min-degree has already converged" cases: no amount of extra
AMD restarts had moved them, and one AMF draw did. Note `pooling_haverly1pq` has
n=31 — [0004](0004-structured-relabelings.md) and the closed-form tests treat that
region as method-limited, and a *different objective* moved it anyway.

## Robustness

The discipline [0004](0004-structured-relabelings.md) introduced, applied to its
successor. `probe_relabel_search`'s columns exist because a heavy-tailed geomean over
300 matrices hands out one-matrix "wins" on request:

| slice | before | after | Δ |
|---|---|---|---|
| full corpus (300) | 0.876925 | 0.871827 | **−0.005098** |
| half A (even index, 150) | 0.872180 | 0.867927 | −0.004253 |
| half B (odd index, 150) | 0.881243 | 0.875532 | −0.005711 |
| drop top-1 contributor (299) | 0.877933 | 0.874262 | −0.003672 |
| drop top-3 (297) | 0.878221 | 0.875494 | −0.002727 |
| drop top-5 (295) | 0.879240 | 0.877128 | −0.002112 |

Same sign and comparable magnitude on both disjoint halves, and still −0.0021 after
deleting the five largest wins (`chp_shorttermplan2d`, `risk2bpb`, `mpbp_15`,
`chimera_k64ising-02`, `chimera_rfr-02`). Contrast 0004's leading policy, which
flipped sign between the halves and died under drop-1. **This is what a real
improvement looks like in these columns**, and it is the reason to trust it will
transfer to a corpus we cannot see: nothing about it is keyed to a matrix, a family, a
size, or an identity — the gate reads `nnz` alone, and the mechanism is a property of
how min-fill and min-degree differ, not a property of this corpus.

## Cap safety

Worst `order()` **0.384 s → 0.439 s** (+0.055 s, +14%) on the same box, against a
2.0 s SIGKILL. The added spend is analytic, not just measured: ≈2.6e-7 s/nnz ×
min(24, 300000/nnz) passes ≈ **0.08 s whatever the matrix looks like**, because the
budget-as-gate bounds passes × nnz by construction. The +0.055 s lands on `arki0016`
(nnz=37208, 0.384→0.439) and `ringpack_30_2` (nnz=121458, near the ceiling) — both
already-won matrices where AMF adds cost without flipping them.

Timing caveat that has bitten this repo before: repeat runs of the same probe vary
~1.6×, so treat 0.439 s as one significant figure, and never extrapolate a *class's*
cost from this corpus to the graded one. The defensible statement is comparative — the
revision this builds on graded successfully at 0.384 s local worst, and this adds an
analytically bounded ~0.08 s on top inside an nnz ceiling.

## Next steps

- **Sweep the relabelled-AMF `dense_alpha`** (α ∈ {0.5, 2.0, 2.5}, mirroring the base
  AMF α sweep). Same reasoning one level down: a different α is a different
  objective, so it is another distinct lottery at no new conceptual risk. Cheap inside
  the existing gate.
- **Raise `RELABEL_AMF_MAX_NNZ`** only with an isolated cost measurement of the
  130k–400k band. The score gain from the band is probably small (few dev matrices
  there) and the cap risk is real, so this is a measure-first item, not a tune-first
  one.
- The generalisation is the point: **any ordering routine whose output depends on the
  input numbering becomes a randomized-restart algorithm under `relabel`, for free.**
  The portfolio contains several that have never been relabelled (the hand-rolled
  RCM / Sloan / ND leaves, MinFill). Each is a candidate second lottery. Prefer the
  ones whose objective differs most from min-degree.
- Do NOT revisit smarter sampling within a single objective — that is
  [0004](0004-structured-relabelings.md), and it is closed.

## Reproducing

```sh
bash scripts/local-candidate-build.sh          # sandboxed candidate rebuild
cargo run --release                            # dev score: 0.871827
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
```

Note the package: `src/ordering/` is compiled only into `ssi-candidate-worker`, so a
probe command without `-p ssi-candidate-worker` silently matches zero tests and exits
green.
