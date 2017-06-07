//! Additional functionality for working with bytes.

/// Extension trait for bit manipulation.
pub trait ByteExt {
    /// Returns whether the byte has its nth bit set.
    fn has_bit_set(&self, n: u8) -> bool;
}

impl ByteExt for u8 {
    fn has_bit_set(&self, n: u8) -> bool {
        if n > 7 {
            panic!("bit {} is out of range for u8", n);
        }

        (self & (1 << n)) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::ByteExt;

    #[test]
    fn has_bit_set() {
        let byte = 0x80;
        assert!(byte.has_bit_set(7));
        assert!(!byte.has_bit_set(0));
    }

    #[test]
    #[should_panic(expected = "bit 8 is out of range for u8")]
    fn bit_out_of_range() {
        0xFF.has_bit_set(8);
    }

}
