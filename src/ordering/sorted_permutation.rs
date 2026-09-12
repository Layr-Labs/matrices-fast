//! Linear construction of the same sorted permuted CSC as feral.
use super::ScoringPattern;

#[cfg(test)]
thread_local! { static VENDOR_BYPASS: std::cell::Cell<bool> = std::cell::Cell::new(false); }
#[cfg(test)]
pub(crate) fn set_vendor_bypass(bypass: bool) { VENDOR_BYPASS.with(|b| b.set(bypass)); }

pub(crate) struct SortedPermutation<'a> {
    pattern: &'a ScoringPattern,
    row_ptr: Vec<usize>,
    columns: Vec<u32>,
    inverse: Vec<u32>,
    cursor: Vec<usize>,
}

impl<'a> SortedPermutation<'a> {
    pub(crate) fn new(pattern: &'a ScoringPattern) -> Self {
        let n = pattern.n;
        assert!(n < u32::MAX as usize);
        #[cfg(test)]
        if VENDOR_BYPASS.with(|b| b.get()) {
            return Self { pattern, row_ptr: Vec::new(), columns: Vec::new(),
                inverse: Vec::new(), cursor: Vec::new() };
        }
        let mut row_ptr = vec![0usize; n + 1];
        for &row in &pattern.row_idx { row_ptr[row + 1] += 1; }
        for row in 0..n { row_ptr[row + 1] += row_ptr[row]; }
        let mut cursor = row_ptr[..n].to_vec();
        let mut columns = vec![0u32; pattern.row_idx.len()];
        for column in 0..n {
            for &row in &pattern.row_idx[pattern.col_ptr[column]..pattern.col_ptr[column + 1]] {
                columns[cursor[row]] = column as u32;
                cursor[row] += 1;
            }
        }
        Self { pattern, row_ptr, columns, inverse: vec![0; n], cursor }
    }

    pub(crate) fn permute(&mut self, permutation: &[usize]) -> ScoringPattern {
        #[cfg(test)]
        if VENDOR_BYPASS.with(|b| b.get()) {
            return super::permute_pattern(self.pattern, permutation);
        }
        let n = self.pattern.n;
        for (new, &old) in permutation.iter().enumerate() { self.inverse[old] = new as u32; }
        let mut col_ptr = vec![0usize; n + 1];
        for (new, &old) in permutation.iter().enumerate() {
            col_ptr[new + 1] = col_ptr[new] + self.pattern.col_ptr[old + 1] - self.pattern.col_ptr[old];
        }
        self.cursor.copy_from_slice(&col_ptr[..n]);
        let mut row_idx = vec![0usize; self.pattern.row_idx.len()];
        // Ascending new rows make each output column sorted without a sort.
        // The stored transpose makes this valid for nonsymmetric input too.
        for (new_row, &old_row) in permutation.iter().enumerate() {
            for &old_column in &self.columns[self.row_ptr[old_row]..self.row_ptr[old_row + 1]] {
                let new_column = self.inverse[old_column as usize] as usize;
                row_idx[self.cursor[new_column]] = new_row;
                self.cursor[new_column] += 1;
            }
        }
        ScoringPattern { n, col_ptr, row_idx }
    }
}
