//! Independent component minima from already scored portfolio candidates.
//!
//! Components never exchange fill. A donor's column-count squares therefore
//! split additively by the original vertex's connected component, regardless
//! of component interleaving. Keep one exact minimum restriction per component
//! while the existing portfolio runs; combine only with the final incumbent.

use std::sync::Mutex;
use super::ScoringPattern;

const NONE: usize = usize::MAX;

pub(crate) struct ComponentMix {
    component: Vec<usize>,
    offsets: Vec<usize>,
    best: Mutex<Best>,
}

struct Best {
    costs: Vec<u64>,
    vertices: Vec<usize>,
}

impl ComponentMix {
    /// Components of fewer than four vertices are chordal and cannot improve
    /// on their AMD restriction. Require two larger components so connected
    /// matrices and a connected graph plus isolated vertices pay no per-donor
    /// work. Construction itself is one linear, iterative graph traversal.
    pub(crate) fn new(sp: &ScoringPattern) -> Option<Self> {
        let n = sp.n;
        let mut seen = vec![false; n];
        let mut component = vec![NONE; n];
        let mut queue = Vec::new();
        let mut offsets = vec![0usize];
        for root in 0..n {
            if seen[root] { continue; }
            queue.clear();
            queue.push(root);
            seen[root] = true;
            let mut head = 0;
            while head < queue.len() {
                let v = queue[head];
                head += 1;
                for &w in &sp.row_idx[sp.col_ptr[v]..sp.col_ptr[v + 1]] {
                    if !seen[w] {
                        seen[w] = true;
                        queue.push(w);
                    }
                }
            }
            if queue.len() < 4 { continue; }
            let id = offsets.len() - 1;
            for &v in &queue { component[v] = id; }
            offsets.push(offsets.last().copied().unwrap() + queue.len());
        }
        let parts = offsets.len() - 1;
        if parts < 2 { return None; }
        let vertices = vec![NONE; *offsets.last().unwrap()];
        Some(Self {
            component,
            offsets,
            best: Mutex::new(Best { costs: vec![u64::MAX; parts], vertices }),
        })
    }

    /// `counts[j]` belongs to original vertex `perm[j]`, not original vertex j.
    /// Call only immediately after the existing scorer has evaluated `perm`.
    /// Exact aliases only need one recording. Lexicographic tie-breaking on
    /// each restriction is associative and independent of worker completion.
    pub(crate) fn record(&self, perm: &[usize], counts: &[i32]) {
        let mut costs = vec![0u64; self.offsets.len() - 1];
        let mut next = self.offsets[..costs.len()].to_vec();
        let mut vertices = vec![NONE; *self.offsets.last().unwrap()];
        for (j, &v) in perm.iter().enumerate() {
            let id = self.component[v];
            if id == NONE { continue; }
            let c = counts[j] as u64;
            costs[id] += c * c;
            vertices[next[id]] = v;
            next[id] += 1;
        }
        let Ok(mut best) = self.best.lock() else { return; };
        for (id, &cost) in costs.iter().enumerate() {
            let range = self.offsets[id]..self.offsets[id + 1];
            if cost < best.costs[id]
                || (cost == best.costs[id]
                    && vertices[range.clone()] < best.vertices[range.clone()])
            {
                best.costs[id] = cost;
                best.vertices[range.clone()].copy_from_slice(&vertices[range]);
            }
        }
    }

    /// No earlier gate or search trajectory sees component mixing. Preserve
    /// the incumbent's interleaving and every non-improved component exactly.
    /// The caller independently verifies the full score before accepting.
    pub(crate) fn refine(
        &self, incumbent: &[usize], counts: &[i32],
    ) -> Option<Vec<usize>> {
        let mut costs = vec![0u64; self.offsets.len() - 1];
        for (j, &v) in incumbent.iter().enumerate() {
            let id = self.component[v];
            if id != NONE {
                let c = counts[j] as u64;
                costs[id] += c * c;
            }
        }
        let Ok(best) = self.best.lock() else { return None; };
        if !costs.iter().enumerate().any(|(id, &c)| best.costs[id] < c) {
            return None;
        }
        let mut next = self.offsets[..costs.len()].to_vec();
        let mut candidate = incumbent.to_vec();
        for v in &mut candidate {
            let id = self.component[*v];
            if id != NONE && best.costs[id] < costs[id] {
                *v = best.vertices[next[id]];
                next[id] += 1;
            }
        }
        Some(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pattern;
    use super::super::scoring_ws::ScoreWorkspace;

    fn fixture() -> (ScoringPattern, Vec<usize>, Vec<usize>) {
        let p = Pattern::from_edges(10, &[
            (0, 1), (0, 2), (0, 3), (4, 5), (4, 6), (4, 7),
        ]);
        let sp = ScoringPattern { n: p.n, col_ptr: p.col_ptr, row_idx: p.row_idx };
        // Opposite good/bad star restrictions, interleaved with two isolates.
        let a = vec![1, 4, 8, 2, 5, 3, 6, 0, 7, 9];
        let b = vec![0, 5, 1, 6, 9, 2, 7, 3, 4, 8];
        (sp, a, b)
    }

    fn record(mix: &ComponentMix, sp: &ScoringPattern, perm: &[usize]) {
        let mut ws = ScoreWorkspace::new(sp.n, sp.row_idx.len());
        ws.flops(sp, perm);
        mix.record(perm, ws.probe_counts());
    }

    fn refine(mix: &ComponentMix, sp: &ScoringPattern, perm: &[usize], ws: &mut ScoreWorkspace) -> Option<Vec<usize>> {
        ws.flops(sp, perm);
        mix.refine(perm, ws.probe_counts())
    }

    #[test]
    fn opposite_component_donors_mix_exactly_and_preserve_isolates() {
        let (sp, a, b) = fixture();
        let mix = ComponentMix::new(&sp).unwrap();
        record(&mix, &sp, &a);
        record(&mix, &sp, &b);
        let mut ws = ScoreWorkspace::new(sp.n, sp.row_idx.len());
        assert_eq!(ws.flops(&sp, &a), 45);
        assert_eq!(ws.flops(&sp, &b), 45);
        let candidate = refine(&mix, &sp, &a, &mut ws).unwrap();
        assert!(super::super::is_bijection(&candidate, sp.n));
        assert_eq!(ws.flops(&sp, &candidate), 28);
        assert_eq!(candidate[2], 8);
        assert_eq!(candidate[9], 9);
        assert!(refine(&mix, &sp, &candidate, &mut ws).is_none());
    }

    #[test]
    fn component_ties_are_independent_of_recording_order_and_threads() {
        let (sp, a, b) = fixture();
        let mut b_tie = b.clone();
        b_tie.swap(1, 3); // Swap two independent leaves of the good second star.
        let forward = ComponentMix::new(&sp).unwrap();
        let reverse = ComponentMix::new(&sp).unwrap();
        for p in [&a, &b, &b_tie] { record(&forward, &sp, p); }
        std::thread::scope(|scope| {
            let reverse_ref = &reverse;
            let sp_ref = &sp;
            let handles: Vec<_> = [&b_tie, &b, &a].into_iter()
                .map(|p| scope.spawn(move || record(reverse_ref, sp_ref, p))).collect();
            for handle in handles { handle.join().unwrap(); }
        });
        let mut ws = ScoreWorkspace::new(sp.n, sp.row_idx.len());
        assert_eq!(refine(&forward, &sp, &a, &mut ws), refine(&reverse, &sp, &a, &mut ws));
    }

    #[test]
    fn collecting_components_preserves_portfolio_results_with_aliases() {
        let (sp, a, b) = fixture();
        let mix = ComponentMix::new(&sp).unwrap();
        let tasks: Vec<super::super::parallel::CandFn<'_>> = [&a, &b, &a, &b]
            .into_iter().map(|perm| {
                let p: Vec<i32> = perm.iter().map(|&v| v as i32).collect();
                Box::new(move || Ok(p.clone())) as super::super::parallel::CandFn<'_>
            }).collect();
        let plain = super::super::parallel::run_candidates(
            &tasks, &sp, sp.n, sp.row_idx.len(), u64::MAX, true,
        );
        let captured = super::super::parallel::run_candidates_mixed(
            &tasks, &sp, sp.n, sp.row_idx.len(), u64::MAX, true, Some(&mix),
        );
        assert_eq!(plain, captured);
        let mut ws = ScoreWorkspace::new(sp.n, sp.row_idx.len());
        let candidate = refine(&mix, &sp, &a, &mut ws).unwrap();
        assert_eq!(ws.flops(&sp, &candidate), 28);
    }

    #[test]
    fn component_partition_skips_connected_empty_and_tiny_components() {
        for p in [
            Pattern::from_edges(0, &[]),
            Pattern::from_edges(10, &[]),
            Pattern::from_edges(10, &[(0, 1), (0, 2), (0, 3), (4, 5), (5, 6)]),
            Pattern::from_edges(8, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7)]),
        ] {
            let sp = ScoringPattern { n: p.n, col_ptr: p.col_ptr, row_idx: p.row_idx };
            assert!(ComponentMix::new(&sp).is_none());
        }
    }
}
