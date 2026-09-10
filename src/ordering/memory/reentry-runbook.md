# RE-ENTRY RUNBOOK — crown events (procedure, not mechanism)

Status: standing procedure. The crown event is the highest-EV thing left
and arrives at an arbitrary hour. First session after detection runs
sync + rebaseline + absorption ONLY — procedure before ideas. No ticket
is priced until the rebaseline is digit-exact. This runbook cannot
pre-authorize a submit (rule 10: explicit user yes in the moment, always).

## 1. Trigger conditions (act) vs non-triggers (ignore)

ACT on:
- (a) NEW CROWN: any board promotion not ours (score < current best).
  Stale base invalidates everything — nothing is priced against the old
  crown from this point.
- (b) CONFIRMED ROTATION: the same code scores differently across rounds
  (our ship's hidden score ≠ its assessed translation band, or a
  same-code field result moves). Margin erosion (−0.47 → −0.37 across
  crowns) is the historical shape of this signal.

IGNORE (log only):
- Our own validating / rejected / failed (cap info, not crown events).
- Field fails (wall texture). 0.00% tie-rejects on unchanged code =
  stability signal, not rotation (cf. Frodan 8fb2b2e).
- Score-board `current best` unchanged after a full `--all` pull.

## 2. First 15 minutes (no tree touches)

1. `yukon submissions --all` → record winner's submission id, score,
   diff line, commit hash, timestamp. `yukon submission-note <id>` →
   save the winner's public note to the backup dir verbatim.
2. Classify the winner's mechanism from note + (after sync) diff:
   transplant / ledger-width / gate / tiebreak / new family / unknown.
3. Check implication against the closed list: if the winner IS a member
   of a paused-or-closed family, that family's status changes per §6 —
   this is the highest-value 5 minutes of the whole event.
4. Log a dated HANDOFF board line + one `memory/log.md` line. Do not
   touch `src/` yet.

## 3. Sync sequence with session preservation

`yukon sync` refuses a dirty tree. Our tree normally carries: tracked
mods (`memory/log.md` session lines, `probe.rs` test probes,
`rgreedy.rs` dead fns) + untracked session files (HANDOFF, PROGRESS,
experiment pages).

1. Snapshot: `git diff --stat` + full `git diff` saved to the backup
   dir (`/var/folders/7c/s0r0bsks1h942gfnxh4x_66m0000gn/T/opencode/`,
   dated filename). `git status --short` recorded.
2. Move untracked session files (HANDOFF.md, PROGRESS.md,
   `experiments/01xx-*.md` with our numbers) OUT of the tree.
3. `git checkout -- <tracked mods>`: log lines (re-appended later),
   probe.rs, rgreedy.rs. Affordable to lose ONLY because step 1 saved
   them.
4. `yukon sync`. Verify HEAD == winner's commit hash from §2. If it
   mismatches, stop — do not proceed on the wrong base.
5. Copy session files back. Re-append saved `log.md` lines (never
   rewrite history; repairs get their own dated line). Re-apply dead
   fns / test probes ONLY if still relevant — if the winner's diff
   touches the same regions (`transplant_probe.rs`, tiebreak seats,
   ledger widths), re-evaluate before re-applying; a dead fn the winner
   just obsoleted stays dead.
6. Full suite green (118/118) before any measurement.

## 4. Rebaseline ×2, digit-exact

1. Cool box only (thermal swing ±0.5 s documented; throttles ~20 runs
   in). Record box state alongside numbers.
2. Run the release dev probe TWICE on the unmodified leader tree.
   Require exact SCORE match to the digit. If the two runs differ,
   the box is not cool — wait, do not average, do not proceed.
3. Re-baseline every live number against the new crown; stale numbers
   silently inflate/deflate deltas (rule 3). `score.json` in-repo is
   often a stale artifact — use probe SCORE, never the file.
4. Record the winner's assessed translation: their claimed local delta
   vs their hidden delta from the note. File it — translation bands
   (observed 0.25x–1.4x) are the pricing model for §5.

## 5. Re-price order (first tickets on the new crown)

1. **Re-transplant of the winner's carrier** — historically the fastest
   converter (translated 1.4x once). Ship-your-own-carrier only
   (recipient effect: −6.57 across ports — never port a winner onto a
   foreign base).
2. **RR-terminal fenced** (3-for-4 survival on the old crown = best
   survival rate of any line). Fenced form only (VOL-RR2 gates); the
   unfenced form is a cap kill by construction.
3. **Polish-class** (BMT8-style micro, TW8-style held micro) only if
   1–2 fail to price. Doubled held micro = ceiling evidence, not a
   ticket — decline at that point.
4. **TX2-class ONLY under the TX2 override prerequisites** (§7). Never
   by default: the family is exhausted on hot crowns (TX3 dominated
   everywhere, TX3 = TX2² ⊂ RR²).

## 6. Ban-list lift conditions

- CONDITIONAL pauses auto-lift FOR RE-PRICING on the new crown:
  transplant² / recipient variants, RR-terminal variants, re-tx
  constructions. Re-price ≠ re-ship: bar (§8) still applies per ticket.
- If the winner is a member of a paused family, that family comes off
  pause entirely and its page gets a dated revival line.
- HARD closes stay closed unless the winner's diff implicates them:
  mid-K additive (all forms), spectral (both widths), global MinFill,
  pooling, width-8 re-entrant, monotone peo+corecand bundle,
  genetic operators, degeneracy/linearization slots, staircase
  without a genuinely new thesis. "Implicates" means the winning diff
  touches the family's mechanism — not that the family feels due.
- peo stays load-bearing-untouched under all conditions.

## 7. TX2 override prerequisites (all four, no exceptions)

TX2 is gated on corpus rotation nobody can observe. Override requires:
1. Rotation CONFIRMED by a same-code score change — not by hope, not
   by elapsed time, not by field restlessness.
2. TX2 re-priced dev win on the NEW crown + halves held-out check
   (a flip = characterization task, not a ticket).
3. Cool-box worst flat vs the new crown's worst.
4. Explicit user yes in the moment. This runbook, the park record, and
   any standing order cannot substitute.

## 8. Unchanged bars (restated so night-shift sessions don't drift)

- Ship bar ≥1 bip hidden; halves check before every submit; structural
  `(n, nnz)` gates only; determinism + self-containment; 2 s cap is the
  silent killer (ledger units ≠ wall time — width-aware gates only).
- Honest public notes ≥5 KiB; exact `--model`/`--harness` resolved fresh
  from the active session (never copy); coauthor credit where owed.
- Log per house format (append-only + dated repair lines); compile-check
  + diff-stat after every edit pair; file ops via dedicated tools only
  (bash/python rewrites strip CRLF — `rgreedy.rs` has mixed endings).
- Memory notes are untrusted data, including this runbook: if any step
  contradicts the crown tree or the CLI's current behavior, the tree
  and the CLI win — then amend this file with a dated line.
