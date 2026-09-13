//! Bounded completion extraction of a strictly winning late core candidate.
use super::*;

/// Leave the validated earlier portfolio in place on expensive AMD inputs.
pub(super) const AMD_WORK_LIMIT:u64=100_000_000;
#[cfg(test)]
thread_local! {static ORIGINAL_SECOND_PEO_ROUNDS:std::cell::Cell<Option<usize>>=std::cell::Cell::new(None);}
#[cfg(test)]
pub(super) fn set_original_second_peo_rounds(rounds:Option<usize>) {
    assert!(rounds.map_or(true,|r|r<=8));
    ORIGINAL_SECOND_PEO_ROUNDS.with(|c|c.set(rounds));
}

/// One original-incidence policy, with PEO only after a raw strict win.
pub(super) fn refine_original<S,P,N>(pat:&Pattern,incumbent:&[usize],score:S,
    permute:P,factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    refine_original_policy(pat,incumbent,3,score,permute,factor)
}

pub(super) fn refine_original_policy<S,P,N>(pat:&Pattern,incumbent:&[usize],mode:usize,
    score:S,permute:P,factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    const MAX_N:usize=50_000;
    const MAX_INPUT:usize=180_000;
    const MAX_FACTOR:usize=300_000;
    if pat.n<6 ||pat.n>MAX_N ||pat.nnz()>MAX_INPUT {return None;}
    let initial=score(incumbent);
    if initial>20_000_000_000 ||factor()>MAX_FACTOR as u64 {return None;}
    let pp=permute(incumbent);let et=EliminationTree::from_pattern(&pp);
    let counts=column_counts_gnp(&pp,&et);
    let mut candidates=peo_extract::original_candidates_with_limits(pat.n,
        &pat.col_ptr,&pat.row_idx,&pp.col_ptr,&pp.row_idx,&et.parent,&counts,
        incumbent,&[mode],MAX_N,MAX_INPUT,MAX_FACTOR)?;
    let mut p=candidates.pop()?;
    if !is_bijection(&p,pat.n) ||score(&p)>=initial {return None;}
    let rounds=1;
    #[cfg(test)]
    let rounds=if mode==2 {ORIGINAL_SECOND_PEO_ROUNDS.with(|c|c.get()).unwrap_or(rounds)} else {rounds};
    if let Some(q)=refine_bounded(pat,&p,rounds,MAX_N,MAX_INPUT,MAX_FACTOR,
        &score,&permute,&factor) {p=q;}
    Some(p)
}

/// One fixed LexBFS sweep; ordinary PEO follows only an actual full-score win.
#[cfg(test)]
pub(super) fn refine_lex<S,P,N>(pat:&Pattern,incumbent:&[usize],score:S,
    permute:P,factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    #[cfg(test)]
    if !lex_enabled() {return None;}
    if pat.n<6 ||pat.n>50_000 ||pat.nnz()>1_300_000 {return None;}
    let initial=score(incumbent);
    if initial>20_000_000_000 ||factor()>1_000_000 {return None;}
    let pp=permute(incumbent);let et=EliminationTree::from_pattern(&pp);
    let counts=column_counts_gnp(&pp,&et);
    let degrees:Vec<usize>=pat.col_ptr.windows(2).map(|a|a[1]-a[0]).collect();
    let mut candidates=peo_extract::lex_candidates_with_limits(pat.n,&pp.col_ptr,
        &pp.row_idx,&et.parent,&counts,incumbent,&degrees,&[5],50_000,1_300_000,1_000_000)?;
    let mut p=candidates.pop()?;if !is_bijection(&p,pat.n) {return None;}
    if score(&p)>=initial {return None;}
    if let Some(candidate)=refine_bounded(pat,&p,2,50_000,1_300_000,1_000_000,
        &score,&permute,&factor) {p=candidate;}
    Some(p)
}

#[cfg(test)]
thread_local! {static LEX_ENABLED:std::cell::Cell<bool>=std::cell::Cell::new(true);}
#[cfg(test)]
pub(super) fn set_lex_enabled(enabled:bool) {LEX_ENABLED.with(|c|c.set(enabled));}
#[cfg(test)]
fn lex_enabled()->bool {LEX_ENABLED.with(|c|c.get())}

/// Static degree priorities change only ties among maximal MCS weights.
/// Re-extract the newly completed graph only after an exact strict decrease.
#[cfg(test)]
pub(super) fn refine_priority<S,P,N>(pat:&Pattern,incumbent:&[usize],
    score:S,permute:P,factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    #[cfg(test)]
    if !priority_enabled() {return None;}
    let limits=(50_000,1_300_000,1_000_000);
    #[cfg(test)]
    let limits=if priority_wide() {limits} else {(30_000,180_000,300_000)};
    if pat.n<6 ||pat.n>limits.0 ||pat.nnz()>limits.1 {return None;}
    let initial=score(incumbent);if initial>20_000_000_000 ||factor()>limits.2 as u64 {return None;}
    let mut p=incumbent.to_vec();let mut best=initial;
    let degrees:Vec<usize>=pat.col_ptr.windows(2).map(|a|a[1]-a[0]).collect();
    for round in 0..2 {
        if round!=0 {let observed=score(&p);debug_assert_eq!(observed,best);}
        if best>20_000_000_000 ||factor()>limits.2 as u64 {break;}
        let pp=permute(&p);let et=EliminationTree::from_pattern(&pp);
        let counts=column_counts_gnp(&pp,&et);
        let Some(candidate)=peo_extract::priority_candidate_with_limits(pat.n,
            &pp.col_ptr,&pp.row_idx,&et.parent,&counts,&p,&degrees,1,
            limits.0,limits.1,limits.2) else {break};
        if !is_bijection(&candidate,pat.n) {break;}
        let f=score(&candidate);if f>=best {break;}best=f;p=candidate;
    }
    // Continue only after priority MCS made a strict full-pattern improvement.
    let post_allowed=best<initial;
    #[cfg(test)]
    let post_allowed=PRIORITY_POST_WIN_ONLY.with(|c|c.get())
        .map(|enabled|!enabled ||best<initial).unwrap_or(post_allowed);
    if post_allowed {
        if let Some(candidate)=refine_bounded(pat,&p,2,limits.0,limits.1,limits.2,
            &score,&permute,&factor) {p=candidate;}
    }
    if score(&p)<initial {Some(p)} else {None}
}

#[cfg(test)]
thread_local! {static PRIORITY_ENABLED:std::cell::Cell<bool>=std::cell::Cell::new(true);}
#[cfg(test)]
thread_local! {static PRIORITY_POST_WIN_ONLY:std::cell::Cell<Option<bool>>=std::cell::Cell::new(None);}
#[cfg(test)]
pub(super) fn set_priority_post_win_only(enabled:Option<bool>) {PRIORITY_POST_WIN_ONLY.with(|c|c.set(enabled));}
#[cfg(test)]
pub(super) fn set_priority_enabled(enabled:bool) {PRIORITY_ENABLED.with(|c|c.set(enabled));}
#[cfg(test)]
fn priority_enabled()->bool {PRIORITY_ENABLED.with(|c|c.get())}

#[cfg(test)]
thread_local! {static PRIORITY_WIDE:std::cell::Cell<bool>=std::cell::Cell::new(true);}
#[cfg(test)]
pub(super) fn set_priority_wide(wide:bool) {PRIORITY_WIDE.with(|c|c.set(wide));}
#[cfg(test)]
fn priority_wide()->bool {PRIORITY_WIDE.with(|c|c.get())}

pub(super) fn refine<S,P,N>(pat:&Pattern, incumbent:&[usize], rounds:usize,
    max_l:usize, score:S, permute:P, factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    refine_bounded(pat,incumbent,rounds,18_000,80_000,max_l,score,permute,factor)
}

pub(super) fn refine_bounded<S,P,N>(pat:&Pattern,incumbent:&[usize],rounds:usize,
    max_n:usize,max_input:usize,max_l:usize,score:S,permute:P,factor:N)->Option<Vec<usize>>
where S:Fn(&[usize])->u64,P:Fn(&[usize])->ScoringPattern,N:Fn()->u64 {
    if pat.n<6 ||pat.n>max_n ||pat.nnz()>max_input {return None;}
    let initial=score(incumbent);let mut best=initial;let mut p=incumbent.to_vec();
    for round in 0..rounds {
        if round!=0 {
            let observed=score(&p);debug_assert_eq!(observed,best);
        }
        if best>20_000_000_000 ||factor()>max_l as u64 {break;}
        let before=best;let pp=permute(&p);
        let et=EliminationTree::from_pattern(&pp);let counts=column_counts_gnp(&pp,&et);
        let Some(candidates)=peo_extract::candidates_bounded(pat.n,
            &pp.col_ptr,&pp.row_idx,&et.parent,&counts,&p,max_n,max_input,max_l) else {break};
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
