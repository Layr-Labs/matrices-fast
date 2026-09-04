# 0057 — Unlabelled AMF α ∈ {0.5, 2.5} inside the existing nnz<130k sweep

- **Date:** 2026-09-04
- **Score:** `0.845281` → **`0.844270`** (−0.001011, −10.11 basis points); fill `0.944707` → **`0.944357`**
- **Status:** **FAILED hidden** as Yukon submission `85dcf2a` (commit `a1da2c0` / local `b3c8d16`). Same class as H3 and R5-32M: CLI status `failed` (not `rejected` on score), almost certainly the hidden 2.0s `order()` cap. Local 300-matrix run was clean at 0.844270 with extra-pass worst +0.041 s, but two extra AMF tickets on the `nnz<130k` envelope did not stay inside the hidden cap. **Do not retry this add. Do not put more `consider()` tickets on this envelope.** Reverted to promoted H2 (`4245c79`) three-alpha sweep `[1.0, 16.0, -1.0]`.

## Hypothesis

The unlabelled AMF sweep already samples `dense_alpha ∈ {1.0, 16.0, −1.0}` (plus default/α5/α2 elsewhere). Rothberg–Eisenstat dense deferral is `threshold = α √n`. Half-integer thresholds 0.5 and 2.5 are different objectives: they defer a different set of coupling rows, so they are a different lottery, not more tickets from the same urn.

Experiment 0005's leftover lead asked for α ∈ {0.5, 2.0, 2.5} on the *relabelled* AMF multi-start. α=2.0 is already in that cycle. A previous session tried *relabelled* extra α 0.5/2.5 only on `n<1000 && nnz<=20k` and got lt_1k noise (0056 ablation 1). That gate is the wrong place: the unique prize, if any, lives where AMF is cheap *and* the graph is large enough for dense-row classification to matter — the 1k_10k / sparse gt_10k band already inside `nnz < 130_000`.

This trial adds the two alphas as **unlabelled** extra `consider()` passes next to the existing three-alpha sweep, same `n < AMF_SWEEP_MAX_N && nnz < 130_000` envelope. No cycle rotation (rotating the five-alpha relabel cycle would steal 5.0 tickets). No nnz-ceiling raise. No extra subtree ops.

## What changed

`src/ordering/mod.rs`, the AMF sweep loop:

```rust
for da in [1.0f64, 16.0, -1.0, 0.5, 2.5] {
```

was `[1.0, 16.0, -1.0]`. Two extra AMF passes, best-of floor, identical envelope.

Test-only: `probe_amf_extra_alpha` in `src/ordering/probe.rs` compares those two passes against the current `order()` incumbent (not against AMD) so a reported win is a ticket the portfolio does not already hold.

## Probe result (release, `--test-threads=1`, this 4-vCPU box)

```
eligible nnz<130k = 279 / 300
unique_wins = 2
score 0.845281 -> 0.844348  delta=-0.000933
order() worst = 1.528s on crudeoil_lee4_10
extra AMF worst = 0.041s on crudeoil_lee4_10
bucket lt_1k   0.893120 -> 0.893120   n=147
bucket 1k_10k  0.868390 -> 0.868331   n=108
bucket gt_10k  0.792071 -> 0.789782   n=45
```

Movers:

| matrix | n | nnz | before | after |
|---|---:|---:|---:|---:|
| `crudeoil_pooling_dt2` | 18742 | 75910 | 0.837535 | **0.735268** |
| `chp_partload` | 5211 | 16740 | 0.816472 | **0.810464** |

lt_1k is a bit-identical control (the n<1000 extra-relabel trial in 0056 was looking in the wrong bucket). gt_10k carries the score: one crudeoil pooling instance is worth most of the 9.33 local bips because the bucket weight is 0.40 over 45 matrices.

## Why this is not H3 / R5

H3 (`21e1d96`, commit `2f1b84a`) widened SqDiv/SqPure to `n<10000 && nnz>=2n` and **failed hidden** (Benchmark exit 1, almost certainly the 2.0s cap). R5-32M (`62253c5`) failed the same way. Those revisions added work on a *larger set of matrices*, including ones near the cap owners.

This revision:

- does not change any subtree round, block, seed, `max_s`, or `SUBTREE_MIN_N`;
- does not widen any `(n, nnz)` gate;
- adds two AMF passes only where AMF is already running three other alphas, all with `nnz < 130_000`;
- measures +0.041 s extra-pass worst, on `crudeoil_lee4_10`, which is already the `order()` worst on this box at 1.528 s. Combined ~1.57 s local vs a 2.0 s kill. H2 (`4245c79`) already passed hidden with this same worst-case matrix in the portfolio (it is inside H2's existing candidate stack; we add 41 ms of AMF, not 4 extra quotient-graph passes on a wider density band).

Search-path overfitting (jtaroreh `a30dc6c`: −4.6 local bips → −0.62 hidden) does not apply: these tickets are complete AMF orderings scored by `consider()` against the exact Σc² objective. They do not become the subtree-search incumbent unless they already beat the current perm on that objective, in which case the later chain starts from a strictly cheaper elimination order.

## Follow-ups

- **Same-count alpha swap** answered in [0058](0058-amf-swap-16-for-half.md): 16→0.5 keeps both unique wins with 0 conservative losses; 16→2.5 is a zero. Do not add a fourth ticket.
- Relabelled-AMF extra α 0.5/2.5 on the *full* `RELABEL_AMF_MAX_NNZ` envelope remains open; do not rotate the existing five-alpha cycle.
- Do not raise the 130k AMF-sweep ceiling without an isolated cost probe of the 130k–400k band.
- Do not resubmit H3, R5, or this extra-ticket add.

## Links

- Open question: sweep relabelled-AMF `dense_alpha` (partially answered: unlabelled 0.5/2.5 inside the 130k sweep is the high-value slice).
- Techniques: [best-of-portfolio.md](../techniques/best-of-portfolio.md), [amd.md](../techniques/amd.md)
- Prior: [0005](0005-relabelled-amf-multistart.md), [0006](0006-cycled-amf-amd-multistart.md), [0056](0056-h3-density2-n10k.md)
