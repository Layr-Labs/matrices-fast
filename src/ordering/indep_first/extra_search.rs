//! Reproject the completed order and search one small retained residual core.
use super::*;
use crate::ordering::{rgreedy,scoring_ws};

pub(super) fn refine<S>(cores:&ExtraCores,incumbent:&[usize],initial:u64,score:S)
    ->Option<Vec<usize>> where S:Fn(&[usize])->u64 {
    let mut best=initial;let mut result=None;let mut searched=false;
    for (amd,lc) in &cores.cores {
        if amd.saturating_mul(2)>initial.saturating_mul(3) {continue;}
        let mut inverse=vec![usize::MAX;incumbent.len()];
        for (i,&v) in lc.il.core_ids.iter().enumerate() {inverse[v]=i;}
        let seed:Vec<usize>=incumbent.iter().filter_map(|&v|
            if inverse[v]!=usize::MAX {Some(inverse[v])} else {None}).collect();
        if !crate::ordering::is_bijection(&seed,lc.core_pat.n) {continue;}
        let mut ws=scoring_ws::ScoreWorkspace::new(lc.core_pat.n,lc.core_pat.row_idx.len());
        let cf=ws.flops(&lc.core_pat,&seed);let factor=ws.nnz_l();
        let projected=splice(&lc.il,&seed);let projected_f=score(&projected);
        debug_assert_eq!(projected_f,cf+lc.il.prefix_flops);
        if projected_f<best {best=projected_f;result=Some(projected);}
        // A fill-free core attains n + 3*edges + 2*triangles. Since the
        // independent prefix has fixed cost, further core search cannot help.
        if factor==(lc.core_pat.n+lc.core_pat.row_idx.len()/2) as u64 {continue;}
        if searched ||lc.core_pat.n>rgreedy::MAX_N ||factor>100_000 {continue;}
        searched=true;
        let w=lc.core_pat.n.div_ceil(64) as u64;let n=lc.core_pat.n as u64;
        let replay=factor.saturating_mul(3*w+22).saturating_add(24*n)
            .saturating_add(2*n*w);
        let allowance=replay.saturating_mul(8).clamp(8_000_000,240_000_000) as i64;
        #[cfg(test)]
        let t=std::time::Instant::now();
        let candidate=rgreedy::search(lc.core_pat.n,&lc.il.core_col_ptr,
            &lc.il.core_row_idx,&seed,cf,allowance,0x9e3779b97f4a7c15);
        if let Some((p,_))=candidate {
            if !crate::ordering::is_bijection(&p,lc.core_pat.n) {continue;}
            let p=splice(&lc.il,&p);
            if !crate::ordering::is_bijection(&p,incumbent.len()) {continue;}
            let f=score(&p);if f<best {best=f;result=Some(p);}
        }
        #[cfg(test)]
        println!("COMPACT_CORE_WORK\t{}\t{}\t{factor}\t{allowance}\t{:.6}\t{best}",
            incumbent.len(),lc.core_pat.n,t.elapsed().as_secs_f64());
    }
    result
}
