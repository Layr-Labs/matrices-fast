//! Preserve the sorted, distinct-score runner-up ledger without doomed clones.
pub(super) fn retain<F>(entries:&mut Vec<(u64,Vec<usize>)>, score:u64,
    permutation:F, capacity:usize)
where F:FnOnce()->Vec<usize> {
    if let Err(position)=entries.binary_search_by_key(&score,|(f,_)|*f) {
        if position<capacity {
            entries.insert(position,(score,permutation()));entries.truncate(capacity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lazy_ledger_matches_stable_reference_including_equal_scores() {
        for capacity in 0..=9 {
            let mut actual=Vec::new();let mut expected=Vec::new();let mut random=0x746d_5b13_c2a9u64;
            for i in 0..2_000 {
                random^=random<<13;random^=random>>7;random^=random<<17;
                let f=if i%3==0 {2_000-i as u64} else {random%41};
                let p=vec![i,capacity];
                retain(&mut actual,f,||p.clone(),capacity);
                expected.push((f,p));expected.sort_by_key(|(f,_)|*f);
                expected.dedup_by_key(|(f,_)|*f);expected.truncate(capacity);
                assert_eq!(actual,expected,"capacity={capacity} event={i}");
            }
        }
        let mut entries=vec![(1,vec![2]),(2,vec![1])];
        retain(&mut entries,2,||panic!("duplicate must not clone"),2);
        retain(&mut entries,3,||panic!("noncompetitive must not clone"),2);
        retain(&mut entries,0,||vec![9],2);
        assert_eq!(entries,vec![(0,vec![9]),(1,vec![2])]);
    }
}
