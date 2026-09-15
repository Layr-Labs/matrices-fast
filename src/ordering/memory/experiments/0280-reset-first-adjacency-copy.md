# 0280 — remove the exact-window game's overwritten adjacency copy

- **Date:** 2026-09-14 (iter70)
- **Base:** `6283436`, the [0278](0278-xch-alloc-and-daily-corpus.md) tree plus the
  documentation-only iter69 commit.
- **Score:** **0.790230 → 0.790230**, all 300 dev-row flop records identical.
- **Status:** **kept, hidden cap still failed** — a score-neutral, output-identical reduction in cap
  exposure. Submitted as `4bb9bdd2` with the tree's bounded value devices; PR #715 was killed by a
  hidden matrix after **175.832 s** of Benchmark-step wall.

## Hypothesis

Every exact-window call obtains a `Game` from the memoized pristine graph. `Game::assemble` copied
the complete `n * ceil(n/64)` bitset from `adj0` into the recycled mutable `adj` buffer. The window
driver then performed a mandatory `game.reset()` before the first possible adjacency read, and
`reset()` copied the same complete bitset again.

The first copy is dead. Removing it changes no logical work charge, sweep, candidate, score, or
gate. It only avoids memory traffic on the family whose mutable bitset is the largest allocation in
the terminal exact-kernel class.

## What changed

`Pristine::game()` became the narrowly named `game_reset_first()`. It calls a private constructor
which may leave the already initialized/recycled `adj` buffer containing arbitrary old words.
`subset_window_descent_config` is its only caller, and its first adjacency operation is the full
`Game::reset`. All general `Game::new` constructors retain eager initialization.

A test build can set `SSI_XCH_EAGER_ADJ=1` to restore the old copy in the same binary. Production
always takes the lazy arm. A duplicate test-only `Instant` construction in `window_dp.rs` was also
removed; it never compiled into production and is not part of the claimed worker delta.

The safety argument covers every exit:

1. `sweeps == 0` is rejected before a game is built.
2. If the first sweep's logical charge cannot be paid, the function returns without reading `adj`.
3. Otherwise `game.reset()` overwrites all `n * w` words before prefix elimination or window
   refinement can read one.

## Result

Fresh same-day full-corpus, graded-closest probes (`SSI_MARK_NOSCORE=1`):

| arm | rows | score | lt_1k | 1k_10k | gt_10k | slowest observed call |
|---|---:|---:|---:|---:|---:|---:|
| eager base | 300 | 0.790230 | 0.8873 | 0.8372 | 0.6822 | 5.573 s |
| lazy candidate | 300 | **0.790230** | 0.8873 | 0.8372 | 0.6822 | 4.812 s |

The 300 `COUNTS` records (`name, n, nnz, AMD flops, returned flops`) diff exactly. The whole-probe
wall is deliberately not used as the wall receipt because the two runs were separate sessions.

The real production-worker frame used distinct eager/lazy binaries, lazy first to reverse the
ordering of the initial screen, three processes per arm and row. Every ratio is identical:

| row | one removed copy/call | eager median | lazy median | delta |
|---|---:|---:|---:|---:|
| `arki0013` | 240.5 MiB | 4.5853 s | 4.3941 s | **−4.2%** |
| `crudeoil_lee4_09` | 30.2 MiB | 4.7092 s | 4.2643 s | **−9.4%** |
| `crudeoil_pooling_dt3` | 112.3 MiB | 3.6635 s | 3.4930 s | **−4.7%** |
| `methanol400` | 68.7 MiB | 2.8530 s | 2.7008 s | **−5.3%** |
| `nd_netgen-3000-1-1-b-b-ns_7` | 131.3 MiB | 3.3001 s | 2.8760 s | **−12.9%** |
| `ringpack_30_2` | 38.7 MiB | 4.5018 s | 3.9397 s | **−12.5%** |

Raw worker permutation files for those six rows were compared byte-for-byte; each eager/lazy pair
had the same SHA-256. The full release suite reports **126 passed / 0 failed / 56 ignored**.

## Hidden receipt

Submission `4bb9bdd2-b486-470b-9ac0-ea91a0c06c99` (submission commit `c876851`, PR
[#715](https://github.com/Layr-Labs/matrices-fast/pull/715), workflow `34852246028`) failed:
`order() exceeded the 2.0s per-matrix cap and was killed`. The Benchmark step began at
13:58:23.064Z and printed the failure at 14:01:18.896Z, **175.832 s** later.

This is much later in the same daily corpus than this lane's earlier 66.6--85.5 s kills. The
mechanism-level reading is limited: reset-first removes real work, but the hidden row that binds the
cap still exceeds two seconds. A later kill is consistent with improved throughput through the
preceding rows, but the redacted row prevents attributing the shift or measuring its own margin.
The result does **not** justify adding a value device; the next candidate still needs another
output-preserving reduction in the terminal tail.

The submission metadata names GPT-5. The user clarified after submission that most development was
performed with DeepSeek V4 Flash; GPT-5/Codex performed final verification, wrote the public note,
and submitted. Yukon had no non-destructive metadata-amendment command, and the submission was not
cancelled at the user's request.

## Why it won

This is the mechanism-level device the daily-rotation regime calls for: no new row is admitted, no
budget is enlarged, and no heuristic or acceptance decision changes. At `n = 44 909`, one dead copy
is 240.5 MiB; a class row can enter the window machinery repeatedly, so avoiding that copy at every
call returns meaningful memory bandwidth without spending any of the score or cap budget.

## Follow-ups

- The nine sparse-span passes still construct and drop nine `Game` objects. A batch entry point
  could reuse one game's auxiliary vectors across the schedule while preserving a reset at the
  start of every pass. It needs a caller scoring callback because only the caller can decide which
  candidate becomes the next seed.
- Do not re-price a value device yet: reset-first's hidden receipt is another cap failure. Continue
  removing output-invariant terminal-tail setup/allocation work first.

## Links

- Evidence: `../evidence/0280-reset-first-adjacency-receipt.txt`,
  `../evidence/0280-remote-verdict-pr715.txt`
- Context: [0277 cap census](0277-cap-margin-census-and-peo-alt-window.md),
  [0278 daily corpus and monotone exchange allocation](0278-xch-alloc-and-daily-corpus.md)
