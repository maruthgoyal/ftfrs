macro_rules! extract_bits {
    ($value:expr, $i:literal, $j:literal) => {{
        const _: () = assert!($i <= $j, "i must be less than or equal to j");
        const _: () = assert!($j < 64, "j must be less than 64");
        const _: () = assert!($i >= 0, "i must be greater than or equal to 0");

        const NUM_BITS: u32 = $j - $i + 1;
        const MASK: u64 = ((1u64 << NUM_BITS) - 1) << $i;

        (($value) & MASK) >> $i
    }};
}

macro_rules! mask_length {
    ($value:expr, $i:expr) => {
        (($value) & (1u64 << ($i)) - 1)
    };
}

pub(crate) use extract_bits;
pub(crate) use mask_length;

#[cfg(test)]
mod tests {

    #[test]
    fn test_extract_single_bit() {
        let value: u64 = 0b1010;
        assert_eq!(extract_bits!(value, 1, 1), 0b1);
        assert_eq!(extract_bits!(value, 0, 0), 0b0);
        assert_eq!(extract_bits!(value, 2, 2), 0b0);
        assert_eq!(extract_bits!(value, 3, 3), 0b1);
    }

    #[test]
    fn test_extract_multiple_bits() {
        let value: u64 = 0b1010_1100;
        assert_eq!(extract_bits!(value, 0, 3), 0b1100);
        assert_eq!(extract_bits!(value, 4, 7), 0b1010);
        assert_eq!(extract_bits!(value, 2, 5), 0b1011);
    }

    #[test]
    fn test_extract_with_large_values() {
        let value: u64 = 0xDEADBEEF00000000;
        assert_eq!(extract_bits!(value, 56, 63), 0xDE);
        assert_eq!(extract_bits!(value, 48, 55), 0xAD);
        assert_eq!(extract_bits!(value, 40, 47), 0xBE);
        assert_eq!(extract_bits!(value, 32, 39), 0xEF);
    }

    #[test]
    fn test_extract_across_byte_boundaries() {
        let value: u64 = 0x00FF00FF00FF00FF;
        assert_eq!(extract_bits!(value, 4, 11), 0xF);
        assert_eq!(extract_bits!(value, 12, 19), 0xF0);
        assert_eq!(extract_bits!(value, 8, 23), 0xFF00);
    }
}
