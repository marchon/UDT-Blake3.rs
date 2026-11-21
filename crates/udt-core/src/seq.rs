//! Sequence-number arithmetic for the UDT data path (31-bit space).

use crate::reliability::SeqNo;

/// UDT data sequence numbers use 31 bits (flag bit reserved).
pub const SEQ_MASK: u32 = 0x7fff_ffff;

/// Mask to the 31-bit sequence space.
pub fn seq_norm(s: SeqNo) -> SeqNo {
    s & SEQ_MASK
}

/// Increment within the 31-bit space.
pub fn seq_add(s: SeqNo, n: u32) -> SeqNo {
    seq_norm(s.wrapping_add(n))
}

/// Distance `b - a` in sequence space (assumes `b` is ahead of `a` by < 2^30).
pub fn seq_offset(a: SeqNo, b: SeqNo) -> u32 {
    seq_norm(b.wrapping_sub(a))
}

/// True if `a` is strictly before `b` in a sliding window of size < 2^30.
pub fn seq_before(a: SeqNo, b: SeqNo) -> bool {
    seq_offset(a, b) != 0 && seq_offset(a, b) < (1 << 30)
}

/// True if `a <= b` in the same window sense (`a == b` or `a` before `b`).
pub fn seq_before_eq(a: SeqNo, b: SeqNo) -> bool {
    a == b || seq_before(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_31_bit() {
        assert_eq!(seq_add(SEQ_MASK, 1), 0);
        assert!(seq_before(10, 20));
        assert!(!seq_before(20, 10));
    }
}
