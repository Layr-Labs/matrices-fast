# 0149 — Declared cap probe: the AMD floor

**Status: measurement instrument, not a score attempt.** This tree cannot
promote and is not meant to. It exists to answer one question that four rounds
of local timing work have not been able to touch.

## The question

The 2 s per-matrix cap, not the score, is what kills submissions in this
benchmark. Across the public Actions history of `Layr-Labs/matrices-fast`,
**62 % of all attempts fail on it**, and the failures are not scattered: 47
consecutive failed runs (2026-09-09 14:36 → 2026-09-10 13:04) all carry the
identical message `order() exceeded the 2.0s per-matrix cap and was killed`,
and all 47 land in a **20 s window at ~18 % of a 405–565 s traversal**. The
grader walks the corpus in file order with no sort (`src/main.rs:243`), so
elapsed time is a monotone position proxy. **The whole field is dying on one
hidden matrix, and it is the same matrix for everyone.**

That row is not in `corpus/dev/patterns.jsonl`. So no local per-row timing can
rank it, and the record shows the local dev tail failing as a predictor three
times in a row — most sharply here:

| tree | 16-vCPU dev worst | rows > 1.0 s | graded outcome |
|---|---:|---:|---|
| frontier `62654a5` | 1.479 s | 17–25 | **passed, three times** |
| `7c12f0a` (ours) | 1.303 s | — | failed at 93.4 s |
| `1b1b7b6` (ours) | 1.191 s | 13 | failed at 92.897 s |

Both of the safer-reading trees died and the slowest-reading tree passes. So
the dev tail is a **refuted** cap instrument, not merely a weak one.

What has never been established is the premise underneath every work-reduction
lead in the queue: **that reducing ordering work reaches the killer row at
all.** Nothing in the record tests it. This experiment does.

## The instrument

`order()` keeps only the two cheap exact certificates and then returns the
grader's own baseline ordering:

```rust
const CAP_PROBE_FLOOR: bool = true;

pub fn order(pattern: &Pattern) -> Vec<usize> {
    if let Some(perm) = forest_certificate(pattern) { return perm; }
    if let Some(perm) = chordal_certificate::order_bounded(
        pattern.n, &pattern.col_ptr, &pattern.row_idx, 2_000_000,
    ) { return perm; }
    if CAP_PROBE_FLOOR { return amd_floor(pattern); }
    /* ... the whole pipeline, unreachable in this tree ... */
}
```

`amd_floor` is `feral_amd::amd_order` with library-default options and nothing
else, with a fallback to the natural order if the i32 conversion, the
`CscPattern` construction or AMD itself fails, so the probe cannot lose a run
to a panic on an unseen pattern. Both branches are bijections of `0..n` and
depend only on `pattern`.

The floor is the right instrument precisely because it is the one tree whose
`order()` cost is *known* to be near-zero — it is the baseline the grader
computes anyway. Nothing about it needs calibrating.

## Local result

Official 300-pattern run, clean, no failures:

| | flop geomean | fill geomean | count |
|---|---:|---:|---:|
| `lt_1k` | 0.9888 | 0.9959 | 147 |
| `1k_10k` | 1.0000 | 1.0000 | 108 |
| `gt_10k` | 1.0000 | 1.0000 | 45 |
| **weighted** | **0.9967** | **0.9988** | 300 |

Exactly as designed: AMD parity everywhere except the `lt_1k` rows the
certificates settle exactly (`st_qpc-m3a` 0.676, and the forest rows). The
whole corpus grades in **13.9 s** of wall clock against roughly 100 s for the
real pipeline.

## What each outcome means

The submission returns two channels, and both are quantitative.

**If it passes.** Ordering work bounds the killer row, and the per-matrix work
ledger direction is validated. More usefully, the **total grader seconds for a
near-zero-work tree is the harness's fixed overhead** — corpus I/O, per-matrix
process spawn, the two required runs, the baseline AMD itself. That number has
never existed, and without it grader wall clock cannot be converted into
ordering seconds. Subtracting it from a real tree's 405–565 s traversal isolates
how much of the grader's clock is actually *our* code, which is the conversion
every cap decision so far has been made without.

**If it fails.** Then no work reduction of any magnitude reaches that row, and
the entire per-matrix-budget program is refuted before it is funded. The cap
program would have to move to memory (`RLIMIT_AS` — the graded run applies a
4 GiB address-space cap that local runs do not), to a non-budgeted kernel, or
to a structural blowup that is not a budget problem at all.

**And the elapsed reading discriminates further.** Tree-to-tree variation in
the pre-killer region is what spreads the field's failures across 86–106 s. So
for a tree this much faster:

- failing **earlier than ~86 s** → the row is genuinely work-bound but even the
  floor exceeds 2 s on it, i.e. the row is large or dense enough that AMD alone
  is the binding cost;
- failing again at **~93 s** → the elapsed clock is dominated by harness
  overhead rather than ordering, and the failure is not ordering work at all;
- **later than ~106 s**, or a pass → the row is cleared.

## Compliance

Deterministic and a pure function of `Pattern`: AMD with default options, no
wall-clock read, no environment, no filesystem, no identity keying and no
windows — there is nothing left in the path to key on. A deliberately
score-losing submission is not a rules violation; `RULES.md` sets no attempt
limit and attaches no penalty to a low score, and a positive-diff tree is
graded and rejected rather than refused.

## Do not confuse this with a closed direction

The floor is **not** a proposal. Nobody should ship AMD parity. The pipeline it
bypasses is worth ~15.7 points of score and stays in the tree behind one
`const`; flipping `CAP_PROBE_FLOOR` to `false` restores `1b1b7b6` exactly.
