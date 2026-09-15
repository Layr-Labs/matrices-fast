# 0288 — remove the retired fork's global suffix scaffolding

- **Date:** 2026-09-15 (iter80)
- **Base:** iter79 `590ea3a` / submission `26f00f80` / PR #745.
- **Change:** remove the fork implementation itself and restore the promoted
  direct suffix control flow and `RefCell` ownership path.
- **Public score:** unchanged from iter79, **0.790253862267**.
- **Status:** verified and ready for submission; production candidate and trusted
  parent builds clean, release suite `127 passed / 0 failed / 56 ignored`.

## The third identical hidden position changes the question

Iter79 removed the bounded dense twin after iter78 had already compiled the
shared-basin fork out of production. That tree was the first composition in
this sequence with neither added search device. It still failed the hidden
two-second per-matrix cap. Workflow `34984482002` entered Benchmark at
`14:55:14.1192976Z` and reported the kill at `14:56:38.4794833Z`, about
**84.360 seconds** later.

The three controlled endpoints now occupy the same position:

| tree | fork state | dense/hub state | Benchmark time to cap |
|---|---|---|---:|
| iter77 | margin 20%, two public movers | bounded 10/3/4 | 84.509 s |
| iter78 | production off | bounded 10/3/4 | 84.681 s |
| iter79 | production off | promoted 8/4/3 | 84.360 s |

Those measurements exonerate the search work performed by both devices on the
binding row. They do not prove that all code introduced to support those
devices disappeared from the production pipeline. Iter79 still carried the
fork's structural refactor around the entire suffix.

## Exact comparison against the promoted base

PR #745 was submitted against promoted commit
`05fa99b2cfc55cc9c7e166d4b348c454d30e6f3c`. Fetching the two ordering source
files from that exact base and comparing them with iter79 showed that the
production fork predicate was constant false, but its infrastructure still
changed every invocation after phase 3:

1. `score_workspace` was consumed with `into_inner()` and rebuilt inside a
   large `run_suffix` closure.
2. The runner-up `RefCell` was consumed and converted to `Arc`, changing every
   late borrow and donor access.
3. The complete phase-4-through-terminal pipeline lived inside a closure with
   six captured or explicit inputs.
4. The function ended in a constant-false fork branch that called the closure,
   rather than continuing directly through the suffix.
5. Stage 4 retained a `run_subtree` condition, before/after flop checkpoint,
   and optional channel notification solely for the suppressed fork lineage.

The optimizer can remove a constant-false branch, but the source-level closure
and ownership conversion are a broader code-generation perturbation than the
admission predicate itself. A two-second hard kill makes that distinction
material: stack layout, inlining, register pressure, and instruction layout can
move a borderline hidden row even when the returned permutation is identical.
No public timing run can reliably price a code-layout effect this small under
the current host noise, so the sound endpoint is exact structural restoration.

## Implementation

Iter80 removes the fork constants and `shared_basin_fork_band` entirely. The
historical test seam is no longer worth keeping in the active source because
commits `a9dad62`, `1a8bb63`, `b380999`, and `75b2cf4` preserve the 0%, 10%,
14%, and 20% forms for reproduction.

The suffix now follows the promoted control flow directly:

- the original scoring workspace remains in its `RefCell` for the whole call;
- the runner-up pool remains in its original `RefCell`;
- stage 4 runs from the current incumbent without a fork-only boolean;
- PEO alternate seeds, transplant donors, and exchange-pool seeds use their
  original scoped borrows;
- `leader_order` returns the finished `best_perm` directly.

This deletes 167 net lines from `mod.rs`. No ordering stage, budget, gate, or
acceptance rule changes. The production dense/hub tuple remains promoted
8/4/3, and the public fork remains absent, so the exact 300-row score is the
iter79 value **0.790253862267**.

## Why this is the narrow next causal test

The promoted base itself completed this daily hidden corpus at 0.840511. After
the two search devices are removed, only a small set of production differences
remains. The direct-suffix restoration is the only one that still spans every
row and every late phase. The other meaningful differences are the iter74 and
iter75 exact-window allocation changes:

- skip a dense adjacency initialization that the mandatory first reset
  overwrites;
- reuse the already materialized sparse adjacency on that same first reset;
- retain one sparse mutable image within the existing one-GiB admission
  envelope between sequential calls.

Those changes were separately checked for identical counts and improved the
measured production-worker wall. Reverting them before removing the global
fork structure would conflate a known allocation win with an unmeasured
whole-pipeline source transformation.

The direct restoration therefore has a crisp hidden interpretation. If iter80
completes, the cap was a code-generation or ownership-layout consequence of the
retired fork scaffolding. If it fails at the same position, the scaffolding is
exonerated and the next controlled endpoint is the exact promoted sparse
allocation path, first removing large-buffer retention and then the reset-first
special case if necessary.

## Expected output invariance

In iter79 production, `shared_basin_fork_band` was an `#[inline(always)]`
function returning false. The only reachable arm invoked `run_suffix` once
with `run_subtree=true`, the current incumbent and deferred candidate, and the
existing scoring workspace. Iter80 executes that same suffix body once with
the same values in direct scope. Removing the unused alternative lineage and
its communication channel cannot change an accepted candidate.

Test builds previously defaulted the fork seam off as well, so the ordinary
release suite and unconfigured corpus probes exercise the same logical path
before and after. Explicit `SSI_SHARED_BASIN_FORK=1` probing is deliberately
retired; its evidence and source remain in the cited commits and experiment
notes.

## Verification plan

1. Build the production candidate through `scripts/local-candidate-build.sh`.
2. Build the trusted parent release target offline and locked.
3. Run the complete `ssi-candidate-worker` release suite.
4. Reproduce the three dense schedule records from iter79 using the exact
   release test binary.
5. Run `git diff --check` and confirm that every changed path remains under
   `src/ordering/`.

## Final verification

The production candidate build and the offline, locked trusted-parent release
build completed successfully. The complete release suite passed:

```
test result: ok. 127 passed; 0 failed; 56 ignored
```

The final focused release probe reproduced the unchanged 8/4/3 records:
c16 Chimera **2,570,031**, `chimera_rfr-02` **2,489,836**, and
`chimera_lga-01` **494,718**. `git diff --check` passed, and the worktree changes
are confined to `src/ordering/`.

## Evidence

- iter79 submission: `26f00f80-4907-404f-a9e0-d3f4bf508c53`
- iter79 hidden receipt: PR #745 / workflow `34984482002`
- exact promoted base used by PR #745:
  `05fa99b2cfc55cc9c7e166d4b348c454d30e6f3c`
- local comparison inputs: `/tmp/pr745-base-mod.rs` and
  `/tmp/pr745-base-rgreedy.rs`
- [0287 bounded dense twin retirement](0287-retire-bounded-dense-twin.md)
- [0286 production fork retirement](0286-retire-basin-fork.md)
- [0283 reset-first and buffer retention](0283-sparse-first-reset-and-buffer-retention.md)

## Links

- [0287 retire the bounded dense twin](0287-retire-bounded-dense-twin.md)
- [0283 sparse first-reset and buffer retention](0283-sparse-first-reset-and-buffer-retention.md)
- [0282 original composed fork](0282-sparse-pristine-fork-twin-composition.md)
