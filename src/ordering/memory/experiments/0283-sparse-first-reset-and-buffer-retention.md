# 0283 — sparse first-reset elision and one-image buffer retention

- **Date:** 2026-09-15 (iter75)
- **Base:** `1a8bb63`, the PR #719 sparse-pristine composition with the shared
  basin fork restored to its 10% AMD-anchor margin.
- **Score:** **0.790142 → 0.790142**; output-preserving cap-margin work.
- **Status:** submitted as `b69938a1` / PR #739; hidden cap failure about
  93.375 s after the final test build. Follow-up: [0284](0284-cap-trim-dense-sweep-and-fork-margin.md).

## Why this follows the two cap failures

The sparse-pristine composition was submitted twice on the 2026-09-14 hidden
corpus. The no-margin fork form (`48e929aa`, PR #725) exceeded the two-second
per-matrix cap after 183.56 seconds of post-build Benchmark wall. Restoring the
fork's measured 10% anchor margin (`c4613ce`, PR #726) retained the public movers
but also cap-failed, this time after 93.79 seconds of post-build Benchmark wall.
The latter source is exactly this experiment's base. Its submission was missing
from the handoff even though the Yukon ledger and public PR both record it.

Those results close another value-device retry on this tree. PR #719 and the
two-image iter72 gate had both completed on the same daily corpus, so the next
change must reduce work while retaining the current permutation decisions. The
new sparse representation has an especially strong invariant to exploit: it
stores CSR as the immutable reset source and only one dense mutable image.

## The two redundant costs

`Pristine::game_reset_first()` calls `Game::new_sparse()` above the 160 MiB
image threshold. The constructor must materialize the entire mutable adjacency
once because its guarded bit insertions also derive the duplicate-insensitive
pristine degree vector. The window driver then performs its mandatory first
`Game::reset()` before any adjacency read. On the inherited implementation that
first reset did this:

1. clear every word of the just-built image;
2. scan the CSR again and reconstruct the same symmetric bits;
3. copy `deg0` and initialize the pivot structures.

Steps 1 and 2 are dead: no operation has changed `adj` between construction and
reset. They are the sparse analogue of iter70's dense overwritten-copy removal,
but were introduced later with PR #719 and therefore escaped that audit.

There is a second lifetime mismatch. `ADJ_POOL_MAX_WORDS` remained fixed at 20
million words (160 MiB), and the same literal was also used as the threshold for
choosing sparse pristine. Thus every sparse game was, by definition, too large
to return its one mutable image to the pool. A row can create games for the
main exchange and subsequent span passes, so each call freed a 190–607 MiB
image and the next call allocated, initialized, and page-faulted it again.

## Change

The implementation makes two narrow, independent changes:

- `Game` carries `sparse_adj_is_pristine`. `new_sparse()` initializes it to
  true; dense constructors initialize it to false. The first reset of a sparse
  game skips only the adjacency clear and CSR reconstruction, flips the flag,
  and executes all other reset bookkeeping unchanged. Later resets still clear
  and reconstruct the image normally.
- The representation threshold stays explicitly fixed at 20 million words,
  while the pool ceiling moves to `1 << 27` words, the exact one-GiB image
  envelope already enforced by `leader_order`. Sequential game calls can now
  exchange ownership of the same allocation through the thread-local pool.

No work-account charge changed. In particular, the first reset still adds
`2*n*w + 8*n` logical operations, even though its physical work is smaller.
The sweep count, component admission, DP order, reservation law, early exits,
and strict exact-flop acceptance are therefore unchanged.

## Correctness argument

The first-reset elision is safe on every control-flow exit:

1. `sweeps == 0`, malformed CSR, invalid seeds, and an unaffordable setup return
   before constructing a game.
2. Once a game exists, `subset_window_descent_config` charges the first sweep
   before calling `reset`. If that charge is refused, it returns without reading
   the adjacency.
3. `new_sparse` fully initializes every word to zero, inserts both directed bits
   for every off-diagonal CSR entry, and increments degrees only when a bit was
   previously absent. Its resulting `adj` is exactly the image the old first
   reset reconstructed.
4. The first reset still initializes `bhead`, `livelist`, `deg`, `known_clique`,
   `nonzero_words`, `mind`, and `nlive` before prefix elimination or window
   refinement can read them.
5. The flag is cleared at that reset. Every subsequent reset follows the old
   clear-and-reconstruct branch, so no mutated fill graph can be reused.

Buffer retention changes ownership duration only. Games never nest on this
path: `Drop` gives the vector to the thread-local pool and the next constructor
takes it back. The pool retains at most one vector per thread. The admitted
maximum remains one mutable image of `1 << 27` words; no second dense pristine
image exists on this representation. At construction, recycled words are all
cleared before CSR insertion, so previous graph state cannot affect output.

The existing sparse-vs-dense trajectory test now also compares the new flag
after reset. It covers scan and bucket pivot modes, cross-word dimensions,
random graphs, duplicate entries, and two-triangle CSR storage.

## Public score and output identity

A fresh 300-row base run completed at **0.790142**. The isolated first-reset
candidate and the final scoped-retention candidate also completed at
**0.790142**. Sorting and diffing every `COUNTS` record (`name, n, nnz, AMD
flops, returned flops`) produced an empty diff: **300 identical, 0 changed**.

The four dev rows above the 160 MiB sparse threshold retain these exact records:

| row | n | nnz | returned flops |
|---|---:|---:|---:|
| `nuclear104` | 39,098 | 257,806 | 78,332,024 |
| `arki0013` | 44,909 | 160,172 | 155,907,534 |
| `transswitch2383wpr` | 59,853 | 277,562 | 3,861,509 |
| `transswitch2736spr` | 69,651 | 331,010 | 7,281,507 |

## Production-worker wall receipt

Two distinct production binaries were built: exact `1a8bb63` and the combined
candidate. `probe_slow_row_stage` staged each pattern and launched real worker
processes, arms interleaved, three repetitions per arm. Ratios are from the
production program rather than the score-heavier probe program.

| row | base median | candidate median | delta | base/candidate ratio |
|---|---:|---:|---:|---:|
| `nuclear104` | 5.4593 s | 5.3778 s | **−1.5%** | 0.758476743 / identical |
| `transswitch2736spr` | 3.3663 s | 3.8997 s | +15.8% | 0.906677223 / identical |
| `transswitch2383wpr` | 3.8908 s | 3.5582 s | **−8.5%** | 0.977069459 / identical |
| `arki0013` | 5.0750 s | 3.8598 s | **−23.9%** | 0.399278668 / identical |
| **aggregate** | **17.7914 s** | **16.6955 s** | **−6.2%** | all identical |

The aggregate is the robust claim: individual rows see enough host noise to
reverse one median, while three rows and the total move in the intended
direction and no row changes its production ratio. An earlier three-repetition
production-worker trial measured all four medians lower and **−4.1%** aggregate.
That precursor had not yet scoped the raised pool ceiling away from unrelated
dense games, so it is supporting evidence rather than the shipping receipt. The
memory audit prompted the final `sparse0.is_some()` restriction; the scoped form
above is the submitted design.

## Cap and memory trade

This change adds no candidate, sweep, component, or logical budget. It cannot
turn a previously skipped row into an admitted one. Its maximum retained vector
is exactly the mutable image that was already live during every sparse game, and
the one-GiB admission law is unchanged. The resource trade is retaining that
allocation between sequential calls instead of returning it to the allocator;
peak adjacency-image count remains one on the sparse path.

The affected public rows are not proof of the redacted hidden failure's identity.
The honest hidden hypothesis is narrower: if a binding row uses sparse pristine,
the candidate removes one redundant full-image reconstruction per game and the
repeated allocation faults between games; otherwise it is output- and nearly
wall-identical to the failed base. A completion would validate the representation
mechanism. Another kill would imply the daily binding row lies elsewhere or still
lacks enough margin, without invalidating the local output proof.

## Evidence

- `.session-backup/iter74-baseline-20260915.log`
- `.session-backup/iter75-sparse-first-reset-full.log`
- `.session-backup/iter75-combined-full.log`
- `.session-backup/iter75-targeted-ab.log`
- `.session-backup/iter75-production-worker-ab.log`
- Public receipts: PR #725 / workflow `34869463384`; PR #726 / workflow
  `34871021560`.

## Links

- [0280 reset-first adjacency copy](0280-reset-first-adjacency-copy.md)
- [0281 terminal class resource law](0281-resource-law-class-gate.md)
- [0282 sparse-pristine composition](0282-sparse-pristine-fork-twin-composition.md)
- [0284 cap trim](0284-cap-trim-dense-sweep-and-fork-margin.md)
