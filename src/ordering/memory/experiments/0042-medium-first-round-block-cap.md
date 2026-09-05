# 0042 — Medium first-round block cap after hidden timeout

**Date:** 2026-09-03

**Frontier:** submission `26932eb`, commit `344a5d2`, hidden **0.874601**.
Its public dev result is **0.849487** with fill **0.947766**.

**Result:** dev **0.849251** (−0.000236), fill **0.947647**.

**Status:** full 300-matrix trusted run passes; submitted for private validation.

## Context: the score win in 0041 was not shippable

Experiment 0041 changed `max_s` from 384 to 256 for the first two subtree-chain
rounds on `1000 <= n < 10000`. It produced a clean public score improvement:
0.849487 to 0.849194, with the small and large buckets unchanged. The full
public `yukon run` also passed every correctness, determinism, purity, memory,
and two-second watchdog check.

Private submission `fd357537-33e6-4eb9-b870-bbc5ff1123a1` nevertheless failed.
The Yukon table only displayed `failed`, so the exact public workflow record was
queried before changing the algorithm:

```sh
gh run view 33788875544 --repo Layr-Labs/matrices-fast --log-failed
```

The benchmark step's terminal message was:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

This distinguishes a resource failure from a bad score, invalid permutation,
nondeterminism, build failure, or policy rejection. The 0041 score mechanism is
still useful, but the 32-block first round does not have enough hidden timing
margin after the block-size change.

## Diagnosis: requested-work ceilings were the wrong abstraction

The original safety argument said that `max_s=256` added no pass, stream, block
limit, or operation budget relative to the frontier. That statement is true
about the configuration constants and false about realized work. Lowering the
largest eligible subtree changes the set of disjoint blocks selected by
`subtree_refine`. A tree with few eligible 384-node blocks can expose many
eligible 256-node blocks, filling more of the 32 slots. The changed first-round
incumbent can also trigger a different sequence of conditional later rounds.

The candidate-worker timing probe made that effect visible. It was run on the
same machine and release profile for both sources:

```sh
cargo test -p ssi-candidate-worker --release -- \
  --ignored --nocapture probe_timing_and_score
```

| source | public score | worst direct `order()` | worst relevant matrix |
|---|---:|---:|---|
| promoted frontier (`max_s=384`) | 0.849487 | 1.081 s | `crudeoil_lee4_10` |
| 0041 (`max_s=256`, 32 first-round blocks) | **0.849194** | **1.606 s** | `rsyn0810m04m` |

On `rsyn0810m04m` specifically, the frontier took 0.608 s and reached ratio
0.8488, while 0041 took 1.606 s and reached 0.8394. The improvement was real,
but the 2.64x runtime jump explains why a same-distribution hidden instance
could cross two seconds. The correct safety target is end-to-end runtime on the
complete portfolio, not only the local search's nominal word-operation ceiling.

## Repair: cap only the first medium round at 12 blocks

The retained score mechanism is the uniform size-bucket rule:

```rust
} else if n < 10_000 {
    cfg.max_s = 256;
    cfg.max_blocks = 12;
    cfg.budget = 1_000_000;
}
```

This selector uses only matrix dimension. It does not inspect matrix names,
hashes, values, corpus position, filesystem state, environment state, or clock
time. Every matrix in the medium tier receives the same deterministic policy.

The scope of the 12-block cap is important. Round 1 consumes
`subtree_cfg_for(n)` directly and therefore uses 12 blocks. Conditional round 2
also inherits `max_s=256`, but deliberately overrides `max_blocks` to 32 in the
existing accepted pipeline. Later rounds retain their explicit 32-block,
512/768-node configurations. This preserves the strong accepted follow-up
search while preventing the smaller initial window from occupying 32 basins and
steering the whole chain into the slow trajectory seen in 0041.

No new candidate, dependency, thread, allocation family, input signal, or
source file is added. The implementation is a three-field medium-tier
configuration change in `src/ordering/mod.rs`; it uses the Rust standard library
and the already reviewed ordering code.

## End-to-end sweep

The sweep varied realized work, not just block size. Every point was measured
through the complete `order()` portfolio on all 300 public matrices.

| medium first-round configuration | score | worst direct call | decision |
|---|---:|---:|---|
| frontier: 384, 32 blocks, 1M | 0.849487 | 1.081 s | accepted control |
| 256, 32 blocks, 1M | **0.849194** | 1.606 s | hidden timeout; reject |
| 256, 16 blocks, 1M | 0.849275 | 1.045 s | safe, positive |
| 256, 16 blocks, 750k | 0.849423 | 1.098 s | too little score margin |
| **256, 12 blocks, 1M** | **0.849251** | **1.079 s** | **ship** |
| 256, 8 blocks, 1M | 0.849268 | 3.588 s | slow downstream basin; reject |

The 8-block point is a useful negative control. Less requested first-round work
does not guarantee a faster complete call: its changed incumbent sent
`pooling_foulds5tp` through a 3.588-second downstream trajectory. Conversely,
the selected 12-block point retains more score than 16 blocks while matching
the promoted frontier's measured worst-case envelope to within 0.002 seconds.

One exploratory edit intended for the medium block count initially matched the
identically named small-tier field. The subsequent source audit caught it; the
small branch was restored exactly to the promoted `16 blocks × 2M` allocation
before the 750k/12/8 measurements and before the submitted source. No small-tier
configuration change is present in this candidate.

## Score attribution

The final trusted result is:

| bucket | frontier | candidate | delta |
|---|---:|---:|---:|
| `lt_1k` (147, weight 0.30) | 0.893893 | **0.893893** | 0 |
| `1k_10k` (108, weight 0.30) | 0.875176 | **0.874387** | −0.000789 |
| `gt_10k` (45, weight 0.40) | 0.796916 | **0.796916** | 0 |
| composite | 0.849487 | **0.849251** | −0.000236 |

The branch condition makes the unchanged outer buckets an attribution control.
The measured values confirm it: only `1k_10k` moves. The selected point retains
about 81% of 0041's aggregate public improvement while eliminating its measured
runtime regression.

The change is not per-instance tuning. The challenge itself defines the three
dimension buckets, and the mechanism applies uniformly within the medium tier:
smaller exact-search subtrees diversify useful local refinements, while a
bounded first round prevents the broader portfolio from entering a costly
conditional trajectory. Hidden matrices drawn from the same distribution see
the same rule.

## Verification

The final 12-block source passed the direct timing and score probe:

```text
lt_1k    0.8939
1k_10k   0.8744
gt_10k   0.7969
SCORE = 0.849251
WORST order() = 1.079 s
```

It then passed the authoritative command:

```sh
yukon run
```

That command rebuilt the candidate under the network-denied dependency sandbox
and invoked `order()` twice on every one of the 300 development matrices under
the hard two-second worker watchdog. Result:

```text
score  0.849251
fill   0.947647
lt1k   0.893893 / fill 0.962420
1k10k  0.874387 / fill 0.959327
gt10k  0.796916 / fill 0.927806
```

There was no timeout, panic, abnormal exit, invalid permutation, determinism
mismatch, purity failure, dependency-policy failure, or memory-cap failure. The
four compiler warnings are inherited dead-code warnings and are unrelated to
this configuration-only change. No dependency manifest or lockfile changed.

## Conclusion

0041 proved that 256-node medium subtrees improve factorization cost but failed
the only gate that matters more than score: hidden runtime. 0042 keeps the same
structural score mechanism and sizes its first-round work from measured
end-to-end behavior. The final candidate improves public dev from 0.849487 to
0.849251 while its worst direct call is effectively identical to the already
promoted frontier on the same machine.

If the hidden result transfers, the next experiment should optimize the
hot-path implementation itself before spending more search budget. If it does
not transfer, preserve this timing lesson: neither smaller blocks nor fewer
blocks are monotonic in complete portfolio runtime because both can change the
conditional incumbent chain.
