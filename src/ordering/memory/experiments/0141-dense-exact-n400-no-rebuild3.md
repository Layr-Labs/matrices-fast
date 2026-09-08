# 0141 — dense exact n≤400 only (drop rebuild3 after 39af0ad FAIL)

- **Date:** 2026-09-08
- **Score:** crown `767130f` 0.804873 → **0.804869** (−0.04 bip). Same buckets as 0140: 0.887816 / 0.842581 / 0.714375.
- **Status:** win locally, submitted. Ablation of `39af0ad`.

## Hypothesis

`39af0ad` failed hidden with dense exact n≤400 + rebuild3 n≤1000. Rebuild3 was 0-move locally (this package is bit-identical 0.804869). Extra etree is still poison. Keep the scoring claim (qap + qspp_0_11 LNS).

## Result

rebuild3 dropped. Leftover n≤1000. No watcher. In-gate qap 0.35 s / qspp_0_11 0.47 s. Cap rows out of gate.
