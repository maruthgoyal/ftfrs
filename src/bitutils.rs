
/// Macro to extract bits from position i to j (inclusive, 0-indexed) from a u64
/// Both i and j must be literals between 0 and 63, with i <= j
#[macro_export]
macro_rules! extract_bits {
    ($value:expr, $i:literal, $j:literal) => {{
        // Compile-time checks
        const _: () = assert!($i <= $j, "i must be less than or equal to j");
        const _: () = assert!($j < 64, "j must be less than 64");
        const _: () = assert!($i >= 0, "i must be greater than or equal to 0");
        
        // Calculate number of bits to extract
        const NUM_BITS: u32 = $j - $i + 1;
        
        // Create mask of the appropriate width and shift it to the right position
        const MASK: u64 = ((1u64 << NUM_BITS) - 1) << $i;
        
        // Apply mask and shift to get the bits in the rightmost position
        (($value) & MASK) >> $i
    }};
}