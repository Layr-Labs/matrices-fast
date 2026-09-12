//! Bounded completion extraction of a strictly winning late core candidate.
use super::*;

pub(super) fn refine<S,P,N>(pat:&Pattern, incumbent:&[usize], rounds:usize,
    max_l:usize, score:S, permute:P, factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    if pat.n<6 ||pat.n>18_000 ||pat.nnz()>80_000 {return None;}
    let initial=score(incumbent);let mut best=initial;let mut p=incumbent.to_vec();
    for round in 0..rounds {
        if round!=0 {
            let observed=score(&p);debug_assert_eq!(observed,best);
        }
        if best>20_000_000_000 ||factor()>max_l as u64 {break;}
        let before=best;let pp=permute(&p);
        let et=EliminationTree::from_pattern(&pp);let counts=column_counts_gnp(&pp,&et);
        let Some(candidates)=peo_extract::candidates_bounded(pat.n,
            &pp.col_ptr,&pp.row_idx,&et.parent,&counts,&p,18_000,80_000,max_l) else {break};
        for candidate in candidates {
            if !is_bijection(&candidate,pat.n) {continue;}
            let f=score(&candidate);if f<best {best=f;p=candidate;}
        }
        if best==before {break;}
    }
    if best<initial {Some(p)} else {None}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn generated_post_core_stress() {
        let mut fixtures=Vec::new();
        fixtures.push(("path18000",Pattern::from_edges(18_000,&(1..18_000).map(|v|(v-1,v)).collect::<Vec<_>>())));
        fixtures.push(("hub16000",Pattern::from_edges(16_000,&(1..16_000).map(|v|(0,v)).collect::<Vec<_>>())));
        for (name,width,height) in [("grid64",64,64),("grid96",96,96),("grid128",128,128),("rectangle32x500",32,500)] {
            let mut edges=Vec::new();
            for v in 0..width*height {
                if v%width+1<width {edges.push((v,v+1));}
                if v+width<width*height {edges.push((v,v+width));}
            }
            fixtures.push((name,Pattern::from_edges(width*height,&edges)));
        }
        let mut materialized=0;let mut above_old_dimension=0;
        for (name,pat) in fixtures {
            let sp=ScoringPattern{n:pat.n,col_ptr:pat.col_ptr.clone(),row_idx:pat.row_idx.clone()};
            let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
            let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
            let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);
            let cp:Vec<i32>=pat.col_ptr.iter().map(|&v|v as i32).collect();
            let ri:Vec<i32>=pat.row_idx.iter().map(|&v|v as i32).collect();
            let core=feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
            let amd:Vec<usize>=feral_amd::amd_order(&core).unwrap().into_iter().map(|v|v as usize).collect();
            for reverse in [false,true] {
                for rounds in [1,2] {
                let mut p=amd.clone();if reverse {p.reverse();}
                let initial=score(&p);let factor=ws.borrow().nnz_l();peo_extract::prof::take();
                let t=std::time::Instant::now();
                let first=refine(&pat,&p,rounds,200_000,&score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l());
                let elapsed=t.elapsed().as_secs_f64();let calls=peo_extract::prof::take().2;
                materialized+=calls;above_old_dimension+=u64::from(pat.n>12_000)*calls;
                let second=refine(&pat,&p,rounds,200_000,&score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l());
                assert_eq!(first,second,"{name}");
                let candidate=first.as_ref().unwrap_or(&p);assert!(is_bijection(candidate,pat.n));
                let f=score(candidate);assert!(f<=initial);
                if factor>200_000 ||initial>20_000_000_000 {assert!(first.is_none());assert_eq!(calls,0);}
                println!("POST_STRESS\t{name}\t{}\t{}\t{rounds}\t{factor}\t{initial}\t{f}\t{calls}\t{elapsed:.6}",pat.n,usize::from(reverse));
                }
            }
        }
        assert!(materialized>0 &&above_old_dimension>0);
        println!("POST_STRESS_TOTAL materialized={materialized} above_old_dimension={above_old_dimension}");
    }
}
