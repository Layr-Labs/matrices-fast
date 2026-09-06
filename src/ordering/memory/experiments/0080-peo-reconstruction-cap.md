# PEO reconstruction cap: measure the refusals before setting the number

## Baseline

Inherited promoted tip `386b89b` (hidden 0.860113), which is our own previous
submission `4bbe2d07`. Local dev baseline on this host, worker rebuilt from
that exact tree: **0.829057**, fill 0.938063 (lt_1k 0.889730 / 1k_10k 0.863434
/ gt_10k 0.757769).

Host caveats, stated up front: `src/ordering` compiles only into
`ssi-candidate-worker`, so `cargo run --release` alone will re-score a stale
worker after an edit and every number here follows `cargo build --release
--workspace`; and this host is few-core and contended, so absolute per-matrix
times run roughly 2x what the runner sees and are used only as deltas.

## The change

One constant. `peo_extract::MAX_LNNZ` goes from `300_000` to `650_000`.

We did not pick that by taste. The terminal PEO block refuses to reconstruct a
completion larger than the cap, and the interesting question is not "is the cap
big enough" but "what is it actually refusing". So we instrumented
`reconstruct` to append a line per accept and per refusal, ran the full 300
matrices once, and counted:

- 630 accepted reconstructions; largest accepted completion **289,121**.
- **6** refusals, all from `MAX_LNNZ`, and they are three matrices seen twice:
  Lnnz **381,126** (n=17,493), **618,374** (n=15,904), **714,536** (n=17,809).
- Zero refusals from `MAX_N` and zero from `MAX_INPUT_NNZ`.

So the cap sat immediately above the largest completion it was letting through
and was excluding the big end of `gt_10k` — the heaviest-weighted bucket —
rather than excluding pathological structure. The instrumentation was reverted
before the measured runs below; the submitted diff is the constant and its
comment.

An honest note on how we got the number: at `800_000` all three matrices enter,
dev reaches 0.828453, and the largest of them, `crudeoil_lee4_10` at Lnnz
714,536, becomes the slowest row on the host at 1.4484 s while contributing the
smallest gain of the three (199,204,080 -> 198,979,463, about 0.11%).
`650_000` admits the two that pay and refuses the one that does not. It keeps
5.97 of the 6.04 basis points and hands back the entire timing cost. We would
rather submit the cheaper 5.97.

## Result

Complete 300-matrix trusted run, baseline then candidate:

| | `386b89b` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.829057 | **0.828460** |
| fill tiebreak | 0.938063 | 0.937826 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863434 | 0.863434 |
| gt_10k | 0.757769 | 0.756278 |

Two strict movers, zero regressions, 298 ties. Exact flop counts:

- `nuclear10a` 62648236 -> 58257464 (7.0%)
- `crudeoil_lee4_09` 164703084 -> 162095061 (1.6%)

Both land in `gt_10k`; the other two buckets are bit-identical, which is what a
cap that only binds on large completions should do.

## Correctness and timing

68 tests pass. No test was added because no code path was added: the block
under the higher cap is the same block, and its acceptance gate — bijection
plus a strictly lower exact score from the independent scorer — is untouched,
so this can only lower a ratio or leave it alone. The extractor's own
per-column checked reconstruction, `checked_add` totals and final
`total + n == lnnz` equality are unchanged, so a raised ceiling does not
weaken any validation; a malformed reconstruction still refuses.

The cost of the cap is linear in the completion it admits. Reconstruction is
O(n + nnz + Lnnz) and each MCS is O(n + |E|), so admitting a 618,374-entry
completion is about twice the work of the 289,121 one already accepted today,
on a matrix whose ordering pipeline is far more expensive than either.

Timing probe on this host, slowest rows first: `multiplants_stg1b` 1.3654 s,
`multiplants_stg1c` 1.2495 s, `multiplants_stg5` 1.2370 s, `chimera_rfr-02`
1.2196 s, `crudeoil_lee4_10` 1.1891 s. Baseline's worst row on the same probe
was 1.3558 s, so the tail is unchanged within this host's noise, and the newly
admitted matrices are not near it.

We are explicit about timing because two earlier submissions, `23af3f88` and
`d713f925`, FAILED hidden validation while scoring fine locally. Both added
unconditional per-matrix work to extra-depth residual cores. That line is
abandoned, and the reason we tightened this cap from 800_000 to 650_000 rather
than keeping the extra 0.07 basis points is the same lesson.

## Also tried, and rejected

Retrying a stalled PEO chain under a reversed zero-weight bucket seed — the
extractor's other free tie choice, keeping the same reconstruction. It produced
**zero** movers across all 300 matrices, so the two existing traversal
directions appear to already exhaust what the tie policy can reach here. Not
submitted, and worth knowing before someone else spends a cycle on it.

## Next

The remaining refusal we can see is `crudeoil_lee4_10`, which needs a cheaper
round rather than a bigger cap. The gate we cannot see through is
`nnz <= 180_000` in the caller, which stops matrices before reconstruction is
ever attempted and so never appears in a refusal count — measuring how many
`gt_10k` matrices die there is the next diagnostic, not the next guess.
