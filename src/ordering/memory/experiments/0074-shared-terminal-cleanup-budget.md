# Share the terminal completion allowance

The previous submission bcccb72a (a1b90b0 locally) failed the hidden two-second
cap despite reusable exact scoring and a fill-free optimality shortcut.

Reserve the independent candidate's 2M cleanup allowance from the inherited
8M allowance whenever that candidate is strictly better before terminal
cleanup. The inherited candidate then receives 6M. If the independent candidate
cannot win before cleanup, strict monotonicity means it cannot win afterward,
so no second cleanup occurs and the inherited candidate retains 8M.
Thus the combined watcher credit allowance is at most 8M instead of 10M.
The extra AMF search and completion reconstruction remain additional work;
this credit bound is not a wall-time guarantee.

The full sandbox benchmark retains all 300 previous scores exactly:
0.832286 versus df6e3f0 at 0.832566; three strict wins, zero losses, 297 ties.
All 66 active tests and 56 generated sandbox stress calls pass. Sequential
public timing totals 71.0764 seconds, maximum 0.6885 seconds, compared with
70.5986/0.6857 for the 10M version. This difference is not a measured speedup;
the rationale is the stricter worst-case work allowance, not that timing series.

This changes the inherited cleanup budget only when a raw independent winner
exists. Consequently there is no general guarantee of matching every hidden
frontier score; exact strict acceptance still ensures no worse than AMD.
Public-corpus nonregression is measured, not claimed as a universal theorem.
