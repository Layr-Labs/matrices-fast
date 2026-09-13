//! Exact, invocation-local reuse of scalar pivot-score powers.
//!
//! Each table has one fixed exponent. A hit returns the result of the same
//! `powf` call made earlier for the same input; no approximation is involved.
//! Storage is bounded independently of the size or density of the graph.

const INTEGER_LIMIT: usize = 4_096;
const FLOAT_SLOTS: usize = 1_024;


pub(super) struct IntegerPower {
    exponent: f64,
    values: Vec<f64>,
}

impl IntegerPower {
    pub(super) fn new(maximum: usize, exponent: f64) -> Self {
        let len = if exponent == 0.0 || exponent == 1.0 {
            0
        } else {
            maximum.saturating_add(1).min(INTEGER_LIMIT)
        };
        Self { exponent, values: vec![f64::NAN; len] }
    }

    #[inline]
    pub(super) fn get(&mut self, input: usize) -> f64 {
        if self.exponent == 1.0 { return input as f64; }
        if self.exponent == 0.0 { return 1.0; }
        let Some(value) = self.values.get_mut(input) else {
            return (input as f64).powf(self.exponent);
        };
        if value.is_nan() {
            *value = (input as f64).powf(self.exponent);
        }
        *value
    }
}

pub(super) struct FloatPower {
    exponent: f64,
    keys: Vec<u64>,
    values: Vec<f64>,
}

impl FloatPower {
    pub(super) fn new(exponent: f64) -> Self {
        let len = if exponent == 0.0 || exponent == 1.0 { 0 } else { FLOAT_SLOTS };
        Self { exponent, keys: vec![u64::MAX; len], values: vec![0.0; len] }
    }

    #[inline]
    pub(super) fn get(&mut self, input: f64) -> f64 {
        if self.exponent == 1.0 { return input; }
        if self.exponent == 0.0 { return 1.0; }
        let bits = input.to_bits();
        let slot = ((bits ^ (bits >> 32)).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 54) as usize;
        if self.keys[slot] != bits {
            self.keys[slot] = bits;
            self.values[slot] = input.powf(self.exponent);
        }
        self.values[slot]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_power_bits_survive_reuse_overflow_and_eviction() {
        for exponent in [-1.0, 0.0, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0] {
            let mut integer = IntegerPower::new(340_000, exponent);
            let mut float = FloatPower::new(exponent);
            // More keys than slots forces collisions, and integer inputs
            // beyond the bounded table exercise the uncached path.
            for pass in 0..3 {
                for index in 0..8_192 {
                    let input = if pass % 2 == 0 { index } else { 8_191 - index };
                    let value = input as f64;
                    let expected = value.powf(exponent).to_bits();
                    assert_eq!(integer.get(input).to_bits(), expected);
                    assert_eq!(float.get(value).to_bits(), expected);
                    let wide = (input as f64 * 1_000_003.0 + 0.25).abs();
                    assert_eq!(float.get(wide).to_bits(), wide.powf(exponent).to_bits());
                }
            }
        }
    }
}
