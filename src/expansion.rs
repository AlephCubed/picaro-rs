use crate::masking::byte_ops::{PICARO_EC, byte_mult};

/// The last 6 columns of the matrix G, transposed.
/// This makes it easier to perform linear combinations in the [`expansion`] function.
const G_LAST_SIX_TRANSPOSED: [[u8; 8]; 6] = [
    [0x1, 0x5, 0x6, 0xC, 0x9, 0x1, 0xA, 0x1],
    [0x1, 0x1, 0x5, 0x6, 0xC, 0x9, 0x1, 0xA],
    [0xA, 0x1, 0x1, 0x5, 0x6, 0xC, 0x9, 0x1],
    [0x1, 0xA, 0x1, 0x1, 0x5, 0x6, 0xC, 0x9],
    [0x9, 0x1, 0xA, 0x1, 0x1, 0x5, 0x6, 0xC],
    [0xC, 0x9, 0x1, 0xA, 0x1, 0x1, 0x5, 0x6],
];

pub fn share_expansion<const SHARE_COUNT: usize>(
    shares: [u64; SHARE_COUNT],
) -> [u128; SHARE_COUNT] {
    let mut result = [0; SHARE_COUNT];

    for i in 0..SHARE_COUNT {
        result[i] = expansion(shares[i]);
    }

    result
}

/// Expands the right side into a 112-bit number.
fn expansion(right: u64) -> u128 {
    let mut result = right as u128;

    // The first 8 bytes remain the same,
    // So we loop over the bytes we need to expand.
    for i in 0..6 {
        let mut sum = 0;

        for j in 0..8 {
            sum ^= byte_mult::<PICARO_EC>((right >> 8 * j) as u8, G_LAST_SIX_TRANSPOSED[i][j]);
        }

        result |= (sum as u128) << 8 * (i + 8);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge_u128, split_u64};
    use rand_chacha::ChaCha20Rng;
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn test_expansion_first_col() {
        let input = 1;

        let output = expansion(input);

        assert_eq!(output, (0x0C09010A0101 << 8 * 8) | input as u128);
    }

    #[test]
    fn test_expansion_all_ones() {
        let input = 0x0101010101010101;

        let output = expansion(input);

        for i in 0..6 {
            let sum = G_LAST_SIX_TRANSPOSED[6 - i - 1]
                .iter()
                .fold(0, |sum: u8, e: &u8| sum ^ *e);

            assert_eq!((output >> 8 * (8 + i)) as u8, sum);
        }
    }

    fn test_masked_expansion<const SHARE_COUNT: usize>(input: u64, rng: &mut ChaCha20Rng) -> u128 {
        let shares = split_u64::<SHARE_COUNT>(input, rng);
        let expanded = share_expansion(shares);
        merge_u128(expanded)
    }

    #[test]
    fn masked() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..256 {
            let unmasked = test_masked_expansion::<1>(i, &mut rng);
            assert_eq!(unmasked, test_masked_expansion::<2>(i, &mut rng));
            assert_eq!(unmasked, test_masked_expansion::<3>(i, &mut rng));
            assert_eq!(unmasked, test_masked_expansion::<4>(i, &mut rng));
        }
    }
}
