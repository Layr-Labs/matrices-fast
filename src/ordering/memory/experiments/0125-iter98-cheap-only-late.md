# 0125 — iter98 cheap-only late (if iter97 FAILs)

## Diagnosis
iter96 lean still FAILed. iter97 drops chain + mid1/cheap3 @0.982s / 19/3.
If iter97 also n/a-fails, mid late (even 1 stream) may still explode under hidden 2×.

## Leap (ready when slot frees)
1. **Strip mid late entirely** (cost>8M band → no streams/descent/simplicial)
2. Keep cheap cost≤8M with **4** streams to hold ≥15 movers
3. Keep shots=1, no chain, mid-core 2, faclay skips

## Target
≥15 movers; worst ≤0.95; fewer heavy paths than iter97.

## Result
Local **0.805573** / worst **1.016s** (noise vs 97) / **19/3**. Timing crown unchanged by late gate.
