# 0190 — the probe frame vs the graded frame, and the ladder's budget curve

Status: **instrument + measurement (no candidate yet; e5a3c6b4 is in flight).**
Iteration 27. Tree: `ada30b4` + a test-only instrument (see below).

## 0. The experiment that was in flight

`~/.angel/loop-experiments/1789208495183-145005/result.json` →
`stop_reason: "setup_error"`, `error: "snapshot requires a regular file <=
16777216 bytes: corpus/dev/patterns.jsonl"`, `patch_path: null`,
`rollout_id: null`. The deep lane died at setup for the second time on the same
cause (the first was 1789201430565-145005). No worker ever ran; no patch
exists. The lane cannot run while the dev corpus is a 99 MB single file, so
"hidden receipts are the only evidence that can rank wide-band shapes" has to
be tested through the ordinary submission path instead.

## 1. The probe frame inflates the biggest rows — measured, not assumed

Every one of the 25 `phase_mark` sites is written

```rust
parallel::phase_mark("1.portfolio", _tph, score(&best_perm));
```

so the argument evaluates **one full scoring pass inside the interval the mark
reports**. `phase_mark` is `#[cfg(test)]` (parallel.rs:355-371) and the graded
worker is `cargo build --release -p ssi-candidate-worker` (no `cfg(test)`), so
that pass does not exist in the graded frame — the memory has said so since
iter25, but never quantified it.

New test-only knob `SSI_MARK_NOSCORE` (src/ordering/mod.rs, `markval!`)
replaces the argument with `best_flops`, which already holds exactly the value
`score(&best_perm)` returns. It changes nothing the ordering does — only the
number printed at the mark.

| probe log | marks | SCORE | WORST order() |
|---|---|---|---|
| 0190-probe-markon.log | `score(&best_perm)` | 0.792212 | 1.155 s |
| 0190-probe-markoff.log | `best_flops` | 0.792212 | 1.153 s |

Both frames: **300/300 identical `final` ratios, 300/300 identical COUNTS** —
the substitution is dev-neutral by construction, so the timing difference is
the mark's own cost (0190-tools/mark_cal.py).

### What the difference is

`acopf_case9241pegase_qcqp`, n=313 068, probe 1.155 s vs graded **0.779 s**
(−33 %). The row's 23 late phases, per-phase, markON → markOFF:

```
2.descent 0.0178 -> 0.0000   5.terminal 0.0182 -> 0.0000  13.alt   0.0174 -> 0.0000
3.search  0.0178 -> 0.0000   8.cleanup  0.0175 -> 0.0000  15.minl  0.0349 -> 0.0000
4.subtree 0.0179 -> 0.0000   9.reduce   0.0896 -> 0.0833  ...all 21 -> 0.0000
```

Every one of those phases is **gated off** on this row (their real work is
below 0.5 ms); the uniform 0.0174-0.0182 s they report is exactly one scoring
pass of a 313k-row pattern. The row's real cost is `1.portfolio` 0.31 + one
`1b.indep` 0.23 + `9.reduce` 0.083 + the rest below measurement.

### Per-row overhead by band (markON − markOFF, 0190-tools/graded_census.py)

| n band | rows | median | max | sum |
|---|---|---|---|---|
| < 1 000 | 147 | −0.0034 | +0.116 | −1.58 |
| 1 000–10 000 | 108 | −0.0068 | +0.041 | −2.29 |
| 10 000–50 000 | 38 | +0.0022 | +0.057 | −0.61 |
| 50 000–100 000 | 2 | +0.0768 | +0.077 | +0.15 |
| ≥ 100 000 | 5 | **+0.254** | **+0.376** | +1.16 |

Only the five ≥ 100k rows carry a systematic overhead (median +0.25 s, i.e.
the probe overstates them by ~30 %); below 50k the per-row noise of two
back-to-back runs on this host (median ≈ 0, p90 ≈ 0.04 s) swamps it.

Consequences for every "worst row" number in the memory: the dev corpus's
graded worst row is **0.78 s**, not 1.13-1.16 s, and the frontier's 1.132 s
`acopf` figure (0151/0182/0184) is a probe-frame number of the same row. The
graded margin on the biggest dev row is therefore ~1.2 s, not ~0.85 s — but
see the caveat below: this does not explain the four ladder kills, because the
killed builds' adds were on rows below 50k, where the two frames agree.

### Negative control: the wide band's chain price is REAL

The 0187 gate (`PEO_ALT_MAX_N` 50 000 → 10 000) was justified by "13.alt costs
0.11-0.20 s/row on 10 000 < n ≤ 50 000". Re-priced in the graded frame, over
the same 38 rows:

```
Sigma 13.alt:  probe 1.656 s (0.0436 s/row)  ->  graded 1.712 s (0.0451 s/row)
```

i.e. the chain's wide-band price is all real work, not mark overhead. The 0187
decision was **not** an artifact of the probe frame — a hypothesis this
measurement kills. (The +3 % is host load: the markOFF run was 2.5 % slower in
corpus total.)

## 2. The ladder's budget curve, at last

Same tree, same graded frame (`SSI_MARK_NOSCORE=1`), only `SSI_TERM_LADDER`
differs; every run is one 300-row corpus pass.

| ladder (budget x seed) | SCORE | vs shipped | extra price |
|---|---|---|---|
| `1e8` A | 0.792316 | +1.04e-4 | −2.2 s corpus |
| `2e8` A (flat, 0166) | 0.792215 | +3e-6 | — |
| shipped density-shaped (5e7 sparse / 2e8 rest) | 0.792212 | 0 | −25 % corpus |
| `1e8` A + `1e8` B | 0.792298 | +8.6e-5 | same as 2e8-A |
| `4e8` A | **0.792204** | −8e-6 | +4.4 s corpus |
| no ladder (frontier 0161) | 0.792436 | +2.24e-4 | — |

Two facts fall out:

1. **The ladder saturates at ~2e8 on seed A.** The second 1e8 of budget on A
   buys 1.0e-4; the next 2e8 buys 1.1e-5 (9 % of that rate) at the same price.
   The mechanism is capped at ~2.3 bips total against the no-ladder build.
2. **Breadth across seeds is not a substitute for depth on one seed.** Moving
   the second 1e8 to seed B (equal total price) is worth 1.8e-5, i.e. **5.6×
   less** than leaving it on A (1.0e-4) — the opposite of the "a second rung is
   nearly worthless / a second seed is worth more" reading of finding 6. That
   finding compared *raw ratio sums*; in score bips, on this corpus, A's own
   second chunk dominates B's first chunk.

So the ladder is closed as a lever: it cannot be improved by re-shaping its
budget or its seeds at this price, and any price cut (1e8) costs
1.0e-4 = 45 % of its whole value.

## 3. Tools added

- `src/ordering/mod.rs` — `markval!` + `SSI_MARK_NOSCORE` (test-only; the
  graded build is byte-identical, 300/300 COUNTS).
- `target/probe-sandbox.sh` — forwards `SSI_MARK_NOSCORE` (target/, never in a
  submission).
- `evidence/0190-tools/mark_cal.py` — per-row mark overhead + graded-frame
  tail table (excludes the `13p.*` counters from the yield column).
- `evidence/0190-tools/graded_census.py` — graded-frame phase census, per-row
  overhead by n band, and the wide-band chain price in both frames.

## 4. Open

- The four ladder kills stay unexplained by construction: the killed builds'
  adds were on rows below 50k, where probe and graded agree to within host
  noise. `e5a3c6b4` (chain 10k-50k at 1e6 units, 0.045 s/row of *real* work)
  is the bisection point and is in flight.
- `1b.indep` on the five ≥ 100k rows: 0.15-0.28 s each, gain on 2 of 5 rows.
  It is the only ≥ 0.2 s item on the rows closest to the cap that is not
  already saturated by a gain.
