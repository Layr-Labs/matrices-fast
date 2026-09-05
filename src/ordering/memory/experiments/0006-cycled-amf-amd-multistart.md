# 0006 — Cycled-AMF and Alternating-AMD multi-start: Diverse objectives at zero marginal time

**Date:** 2026-09-02
**Score:** 0.871827 → **0.871434** (−0.000393, −4.5 bips). Buckets: lt_1k 0.9063→0.9064,
1k_10k **0.8991→0.8971** (−0.0020), gt_10k 0.8255→0.8260. Fill tiebreak 0.960774→0.960579.
**Status:** WIN, shipped.

## Hypothesis

Following [0005](0005-relabelled-amf-multistart.md)'s finding that diversity of elimination objective beats spending budget on redundant draws from the same distribution, we investigate the open questions in `memory/open-questions.md`:

1. **Relabelled-AMF `dense_alpha` cycling**: In 0005, relabelled AMF was shipped with a static `dense_alpha = 5.0` for all restarts. However, different dense detection thresholds ($\alpha \in \{5.0, 2.0, -1.0, 1.0, 16.0\}$) represent fundamentally distinct objective landscapes for the elimination heuristic. Instead of drawing every AMF ticket from the $\alpha = 5.0$ distribution, cycling $\alpha$ across restarts samples across 5 distinct objective distributions at **exactly zero marginal wall-clock time**.
2. **Alternating AMD aggressive mode**: In the relabelled AMD loop, alternating restarts between aggressive AMD (`aggressive: true`) and non-aggressive AMD (`aggressive: false`) samples two different tie-breaking cascades without requiring extra passes or extra time.
3. **Small-graph dense-disabled dual evaluation**: On small graphs ($n < 5000$), evaluating the dense-disabled AMF ($\alpha = -1.0$) on the same relabelled core $B = Q A Q^T$ reuses the permutation allocation and captures dense coupling interactions.

## Results & Verification

- Baseline score: **0.871827** (fill tiebreak 0.960774)
- New score: **0.871434** (fill tiebreak 0.960579)
- Net improvement: **−0.000393** (4.5 bips), exceeding the 1 bip minimum threshold.
- Measured worst-case `order()` time: **0.531 s** (well below the 2.000 s SIGKILL cap and well below the 1.019 s historical threshold).
- All determinism and bijection invariants pass.
