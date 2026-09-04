# 0057 — Bootstrap variance: the dev score's noise floor

## Status

Methodology result (no production change). New test-only probe
`probe_bootstrap` in `src/ordering/probe.rs` (never shipped, never read by the
grader): runs shipped `order()` once per matrix, records per-matrix ln(ratio)
by bucket, resamples *within* buckets (fixed 147/108/45 counts, deterministic
xorshift stream, B=2000 replicates), and re-aggregates with the
harness-exact `aggregate`. Also reports drop-top-k and the top-win list.

## Result on the H2 frontier (dev 0.845281)

```text
OBSERVED score = 0.845281
drop-top-1 score = 0.855589   (+103 bips from removing ONE matrix)
drop-top-3 score = 0.859688   (+144 bips)
BOOTSTRAP B=2000 mean = 0.845715  95% CI = [0.811525, 0.879873]
CI half-width = 0.034174 (404 bips)
```

Top wins: `pooling_sppc1pq` (gt_10k, 0.1936), `maxcsp-langford-3-11`
(lt_1k, 0.3305), `pooling_sppa0pq` (1k_10k, 0.3434), `waterund14` (0.3686),
`ringpack_30_2`, `pooling_sppb5pq`, `mpbp_35`, `multiplants_stg5`.

## Interpretation (read carefully — level CI ≠ delta CI)

The CI above is for the score *level* under corpus resampling: it is enormous
(±404 bips) because a handful of giant wins dominate the geomean — removing
`pooling_sppc1pq` alone moves the score 103 bips. This does **not** mean every
1-bip delta is meaningless: candidate comparisons on the *same* corpus are
paired (giant wins largely cancel). But it proves three things:

1. **Breadth beats magnitude.** A win concentrated in 1–2 matrices
   (max_sub's `gams05`/`pooling_sppc3pq`, which failed hidden at +0.04 bips)
   is indistinguishable from corpus luck. H2's shape — two buckets moving
   with tie counts nearly flat (non-tie polish across families) — is what a
   real improvement looks like here.
2. **Drop-top is mandatory for sub-2-bip deltas.** Any future candidate under
   ~2 bips must report drop-top-1/drop-top-3: if the gain vanishes, it is one
   matrix and must not ship (0004's rule, now quantified: one matrix can be
   worth 100+ bips).
3. **The hidden refresh cuts both ways.** The hidden corpus redraws the tail:
   giant dev wins may not exist there (magnitude risk), but broad small wins
   transfer at ~1× (H2: 1.07×). Prefer interventions whose dev gain is spread
   over ≥5 matrices in ≥2 buckets.

## Rule for the autoresearch loop (adopted)

- >2 bips + multi-bucket + drop-top-3 retains ≥60% of gain → submittable
  (timing permitting).
- 1–2 bips → submittable only with drop-top robustness AND timing at/below
  parent worst (the stack met the second, and failed the first test on
  hidden timing — margin too thin to survive).
- <1 bip → never submittable regardless of shape (benchmark bar).

## Verification

`cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored
--nocapture --test-threads=1 probe_bootstrap` — deterministic (fixed seed);
re-run reproduces the CI exactly. Production code untouched (only
`probe.rs`); full `cargo test` suite green.
