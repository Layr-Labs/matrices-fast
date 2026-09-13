# 0218 — the probe's test default is NOT the graded program on the four largest rows

- **Date:** 2026-09-12 (iter45)
- **Score:** test default 0.791851 → production mirror **0.791635** (same binary, same
  session, one env var); worst `order()` 1.107 → 1.160 s
- **Status:** measured, shipped behaviour already carries it (no code change needed);
  the finding corrects the *frame* every future dev measurement must use

## Hypothesis

The iter44 audit left one unexplained object: the promoted source (`d0125f1`, board
leader `fe4f40c`) reads **0.791635** in the record but **0.791851** when rebuilt, and the
gap is exactly four rows in the top-weight bucket (crudeoil_pooling_dt3,
gabriel09, gasprod_sarawak81, popdynm200 — n = 21 688…30 660). Hypothesis: the two
readings are two *frames* of the same program, not two programs — specifically that a
`#[cfg(test)]` seam whose default differs from the `cfg(not(test))` production value
separates them.

## What changed

Nothing in production. A test-only seam `SSI_INDEP_FORCE_N` was added so one binary can
re-point the stage-1b *force* gate (`INDEP_FORCE_MIN_N`, `mod.rs`), and the audit below
was run on the unchanged tree.

## Result

Same binary, one session, 300 dev rows, `cargo test --release -p ssi-candidate-worker`:

| arm (env) | SCORE | WORST order() |
|---|---|---|
| none (test default) | 0.791851 | 1.107 s |
| `SSI_INDEP_FORCE=1` (production mirror) | **0.791635** | 1.160 s |
| `SSI_INDEP_FORCE=1 SSI_INDEP_FORCE_N=40000` (arm cannot fire — internal control) | 0.791851 | 1.116 s |
| `SSI_INDEP_FORCE=1 SSI_INDEP_FORCE_N=20000` (= production constant) | 0.791635 | 1.173 s |
| `SSI_INDEP_FORCE=1 SSI_INDEP_FORCE_N=8000` | 0.792744 | 1.299 s |
| `SSI_INDEP_FORCE=1 SSI_EXCHANGE_LEDGER=1073741824` | 0.791634 | 1.393 s |
| `SSI_INDEP_FORCE=1 SSI_PEO_ROUNDS=5` | 0.791638 | 1.319 s |

Per-row: the test-default and production-mirror tables differ on **exactly the same four
rows, with exactly the values** the historical logs dispute — crudeoil 0.7100/0.6919,
gabriel09 0.8922/0.8976, gasprod 0.9199/0.9135, popdynm 0.9572/0.9489 — i.e. the
"unreproducible 0.791635" of `0209b` is the *production* frame and the "pinned 0.7100"
of `0215`/`0217` is the *test* frame. The control arm (`F_N=40000`) reproduces the test
default to the digit, so the separation is the gate, not a stale binary.

## Why it won / lost

`mod.rs` has two sides of the stage-1b force-adoption switch:

- `#[cfg(not(test))] fn indep_force_off() -> bool { false }` — the graded build forces
  the independent-set lift at stage 1b on every row with `n >= INDEP_FORCE_MIN_N`
  (20 000), so the *subtree* stage polishes the lift instead of the portfolio incumbent.
- `#[cfg(test)] fn indep_force_off() -> bool { std::env::var("SSI_INDEP_FORCE").is_err() }`
  — the probe default is force-OFF everywhere, so those rows go down the *deferred*
  path (raw lift compared against the subtree-polished portfolio incumbent at 4b).

On crudeoil the two trajectories diverge: the lift (0.7335) is only 2.7 % ahead of the
portfolio incumbent, so the 10 % immediate margin does not clear and the deferred rule
drops the lift after the subtree stage polishes the *other* incumbent to 0.7370
(final 0.7100); with force on, the subtree polishes the *lift* to 0.6927 (final 0.6919).
Dev value of the switch: **2.16e-4** (0.791851 → 0.791635), 3 wins / 1 loss, entirely in
the 0.40-weight `gt_10k` bucket.

Consequences recorded:

1. Every absolute dev number must be labelled with its frame. The record's iter43 chain
   (0.791693 → 0.791635) was a *production-frame* chain; the iter44 restoration runs
   (0.791851, 0.791788, 0.791766) were *test-frame* runs. Neither "does not reproduce" —
   they are different programs by construction. The submitted build's dev value is
   **0.791635**, not 0.791851.
2. Any device whose value lands on rows with n ≥ 20 000 must be priced with
   `SSI_INDEP_FORCE=1`; everything else is frame-invariant (the class gates end at
   n ≤ 12 000).
3. The open lead "a stage present at 16:50 owned the four above-gate rows (1.8e-2 on
   one row)" is closed: that stage is the production force arm, it is present in the
   shipped build, and its dev value is the 2.16e-4 above.
4. The gate is load-bearing: lowering it to 8 000 (which re-admits the ~35 mid-size
   lift rows on the forced trajectory) costs **1.1e-3**, and raising it out of range
   (40 000) costs the full 2.16e-4. 20 000 stays.
5. The record's own "next bats" (lead 24) are dead in the graded frame: ledger
   512M → 1G buys −1e-6 for +0.22 s of peak, PEO 4 → 5 buys +3e-6 for +0.15 s.

## Follow-ups

- Next-bat candidate that the frame finding *enables*: the deferred rule compares a
  **raw** lift against a **subtree-polished** incumbent. On the four rows where force
  fires, polishing the lift is what wins (0.7370 → 0.6927). The symmetric device — polish
  the deferred lift too and compare like with like at 4b — is unmeasured; the subtree
  stage is a 5-deep inline cascade (`rgreedy::subtree_refine`), so this needs the cascade
  factored into a callable polish (one extra pass on ~35 lift rows) before it can be
  priced.
- Cheaper structural variant: the force gate is a proxy for "the residual core is a
  mesh-like Schur complement". `IndepLift::core_n()` already exists; exposing it from
  `run()` and gating on the core (rather than on `n`) is the next monotone predicate to
  sweep in the production frame.

## Links

- Techniques: [../techniques/](../techniques/)
- Ledger: [../evidence/0162-remote-submission-ledger.txt](../evidence/0162-remote-submission-ledger.txt)
- Evidence: `0218-frame-testdefault-300.log`, `0218-frame-indepforce-300.log`,
  `0218-forceN-40000.log`, `0218-forceN-20000.log`, `0218-forceN-8000.log`,
  `0218-prod-ledger1G.log`, `0218-prod-peo5.log`
