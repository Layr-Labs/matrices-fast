# Experiment 0053: selective lower-medium round-4 depth

## Status

Submitted for hidden validation. Starting from the official leader at promoted
commit `7177486`, this experiment changes one functional expression: chain
round 4 receives a 64M per-block search budget only when
`1,000 <= n < 6,000`; every other matrix keeps the hidden-proven 32M budget.

The candidate development result is:

```text
score = 0.845469
fill  = 0.944729
```

The accepted parent scored 0.845707 / 0.944811 locally and 0.871418 /
0.955486 on the hidden corpus. The public reduction is 0.000238, about 2.81
basis points relative to the parent.

## Context: use the failed control instead of discarding it

Experiment 0052 doubled round 4 from 32M to 64M globally. It was a strong
score control:

```text
32M globally: 0.845707
64M globally: 0.845411
delta:         -0.000296
```

But its submission `de541fe9-909c-4725-8792-c1f344cb2299` failed because one
hidden matrix exceeded the hard 2-second `order()` cap. That failure closes a
global or larger raw-budget increase. It does not imply that every matrix
needs the same cap. The public diagnostics from 0052 separated the effect by
size bucket:

| bucket | accepted 32M | global 64M | delta |
|---|---:|---:|---:|
| lt_1k | 0.893224 | 0.893334 | +0.000110 |
| 1k_10k | 0.869703 | 0.868807 | -0.000896 |
| gt_10k | 0.792071 | 0.791922 | -0.000149 |

The small bucket actively regressed, while nearly all of the weighted public
gain came from the medium bucket. The large bucket improved slightly but also
contains the largest bitset subproblems and the highest observed runtime.
Therefore the failed run provides a direct allocation signal: spend deeper
round-4 work only inside the score-dense, cheaper medium region.

## Cutoff selection

The first selective probe used `1,000 <= n < 6,000`. This is not a matrix-name
lookup and does not depend on corpus position or values. It is a simple
dimension gate available from the input pattern and covers the public movers
that supplied most of 0052's gain, including lower-medium crude-oil, pooling,
chimera, nuclear, and synthetic-network cases.

The cutoff deliberately stops before the top 40% of the medium bucket. That
upper band contains the previous medium worst case (`arki0016`, n=7993) and
would buy only a small additional aggregate reduction. On the public corpus,
extending 64M to all matrices below 10,000 yields medium geomean 0.868807;
stopping below 6,000 yields 0.868912. Thus the wider gate buys only 0.000105
inside one 0.30-weight bucket, approximately 0.37 overall basis points, while
placing substantially larger matrices on the unaccepted depth. That trade is
not justified after the exact 0052 timeout.

No `nnz` threshold is needed for the selected point. The `n < 6,000` gate is
the stronger direct bound on the dense bitset search state, and the measured
worst call confirms that the resulting realized work remains close to the
accepted runtime.

## Implementation

The only functional change from promoted commit `7177486` is:

```rust
cfg4.budget = if (1_000..6_000).contains(&n) {
    64_000_000
} else {
    32_000_000
};
```

All other chain parameters remain exactly as hidden-accepted:

- rounds 2 and 3: 8M per block;
- round 4: seed/round 3, 32 blocks, `min_s = 16`, `max_s = 768`;
- round 5: seed/round 4, 32 blocks, 16M per block;
- terminal primary seed 5 and the accepted conditional follow-up seeds 6/7;
- every family, gate, incumbent comparison, and final permutation check.

Round 4 is still reached only after the preceding subtree rounds each find a
strictly better incumbent. Every proposed local ordering is independently
re-scored by the exact flop evaluator and accepted only on a strict reduction.
The larger budget therefore changes which candidates can be found, not the
objective, validity checks, or best-of safety rule.

## Focused score result

The selected lower-medium gate produced:

| bucket | accepted source | selective 64M | change |
|---|---:|---:|---:|
| lt_1k | 0.893224 | 0.893224 | 0 |
| 1k_10k | 0.869703 | 0.868912 | -0.000791 |
| gt_10k | 0.792071 | 0.792071 | 0 |
| weighted score | 0.845707 | **0.845469** | **-0.000238** |

The unchanged small and large bucket results are controls: the new branch is
unreachable there. The full-score improvement is exactly the medium change
times its 0.30 bucket weight, modulo the scorer's retained precision.

Several public lower-medium cases move materially, but not uniformly. Examples
include `crudeoil_pooling_ct3` from about 0.6511 to 0.6258,
`crudeoil_lee1_07` from about 0.8035 to 0.7882, and
`chimera_mgw-c16-2031-01` from about 0.8187 to 0.7950. Some cases select a
different but slightly worse 64M trajectory than the parent. The corpus-level
comparison, rather than cherry-picked wins, is the decision criterion.

## Runtime and hidden-risk argument

The focused 108-matrix medium probe reported:

```text
accepted 32M worst = 0.644 s
selective 64M worst = 0.661 s
selective total     = 34.1 s
```

The selected candidate's worst observed call is only 0.017 seconds above the
accepted source in these measurements. By comparison, the global 64M control
reported bucket worsts of 0.376 / 0.739 / 1.003 seconds and then failed hidden
timing. The new candidate removes 64M from both the entire large bucket and the
upper-medium range containing the public medium worst. It also avoids the
score-negative small bucket.

This does not prove that an unknown hidden matrix cannot be pathological, but
it is a materially different timing hypothesis from 0052:

- no additional stage, block, stream, task, or matrix gate is added;
- the 64M depth is confined below 6,000 vertices;
- all `n >= 6,000` matrices execute the already promoted 32M path;
- the public worst stays close to the accepted parent's measured envelope;
- the expected improvement exceeds the one-basis-point promotion threshold by
  almost threefold.

This is the narrowest gate that recovers most of the failed control's score
gain while retaining strong empirical timing evidence.

## Nearby cost-neutral seed controls

Before increasing depth selectively, equal-work deterministic seed variants
were checked so that a free diversification win would be preferred if one
existed. None cleared the promotion threshold:

- terminal primary seeds 4, 6, and 8 all regressed versus accepted seed 5;
- chain round-4 seeds 2, 5, and 6 improved by at most about 0.41 bp;
- chain round-5 seed 5 tied the parent flop score at 0.845707 and worsened fill;
- chain round-5 seed 6 regressed to a projected 0.845744;
- terminal follow-up seed 8 improved only about 0.23 bp.

Those controls support a depth reallocation rather than more seed tuning. They
also leave the submitted source on every accepted deterministic seed.

## Verification

Run from the benchmark work directory on the Mac Studio:

```sh
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

The release unit suite completed with 25 passed, 0 failed, and 16 ignored. The
full trusted 300-matrix command completed successfully and includes the
sandboxed candidate build, source scan, pinned dependency/license checks,
optimized execution, determinism repetition, permutation validation, watchdog,
and trusted factorization-cost scoring.

Exact full result:

```text
OK  0.845469  0.944729
```

The local workstation and Mac Studio copies of `src/ordering/mod.rs` matched
before submission:

```text
sha256 3f31452d21a53110181611aad1a869811d98999692e02fdbd9c0f388b50be4e3
```

## Rule compliance

- All edits are under `src/ordering/`.
- The entrypoint remains `pub fn order(pattern: &Pattern) -> Vec<usize>`.
- The algorithm sees only the supplied sparsity pattern.
- The returned result is a deterministic permutation.
- Rust standard library and the challenge's already allowed modules only.
- No dependency, manifest, lockfile, build script, environment, network,
  subprocess, clock, filesystem, FFI, or thread-count changes.
- No matrix-name checks, stored answers, corpus-position checks, or hidden
  metadata.
- The trusted grader recomputes the objective from the returned ordering.

## Decision and stopping rule

Submit because the candidate passes every public gate, improves the primary
score by 2.81 basis points, slightly improves fill, and keeps observed runtime
near the hidden-accepted parent. If hidden validation promotes it, the next
work should continue with fixed-work allocation or a new structural heuristic,
not widen this 64M cutoff casually. If it times out, restore global 32M and
close 64M round-4 depth entirely; another cutoff retry would be overfitting the
public timing sample rather than following robust evidence.
