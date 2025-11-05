use crate::masking::byte_ops::{byte_mult, PICARO_EC};

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

pub(crate) fn share_expansion<const SHARE_COUNT: usize>(
    shares: [u64; SHARE_COUNT],
) -> [u128; SHARE_COUNT] {
    let mut result = core::array::from_fn(|_| u128::default());

    for i in 0..SHARE_COUNT {
        result[i] = expansion(shares[i]);
    }

    result
}

// Todo Performance might be bad.
/// Expands the right side into a 112-bit number.
pub(crate) fn expansion(right: u64) -> u128 {
    let bytes = right.to_be_bytes();
    let mut result = ((right as u128) << 48).to_be_bytes();

    // The first 8 bytes remain the same,
    // So we loop over the bytes we need to expand.
    for i in 0..6 {
        // Perform linear combination on pre-transposed matrix.
        result[8 + 2 + i] = bytes
            .iter()
            .zip(G_LAST_SIX_TRANSPOSED[i])
            .map(|(byte, g)| byte_mult::<PICARO_EC>(*byte, g))
            .reduce(|sum, e| sum ^ e)
            .expect("There will always be 8 elements");
    }

    u128::from_be_bytes(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge_u128, split_u64};
    use rand_chacha::rand_core::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    #[ignore] //Todo
    fn test_expansion() {
        let right = u64::from_be_bytes([0, 1, 2, 3, 4, 5, 6, 7]);
        let result = expansion(right);

        // Constants to align the expected grid.
        const A: u8 = 0xa;
        const C: u8 = 0xC;

        #[rustfmt::skip]
        assert_eq!(
            result.to_be_bytes(),
            [
                // Result is only 112-bits, so the first two bytes are zero.
                0, 0,
                // Next 8 bytes remain the same.
                0, 1, 2, 3, 4, 5, 6, 7,
                // Then the linear combinations.
                1*0 + 5*1 + 6*2 + C*3 + 9*4 + 1*5 + A*6 + 1*7,
                1*0 + 1*1 + 5*2 + 6*3 + C*4 + 9*5 + 1*6 + A*7,
                A*0 + 1*1 + 1*2 + 5*3 + 6*4 + C*5 + 9*6 + 1*7,
                1*0 + A*1 + 1*2 + 1*3 + 5*4 + 6*5 + C*6 + 9*7,
                9*0 + 1*1 + A*2 + 1*3 + 1*4 + 5*5 + 6*6 + C*7,
                C*0 + 9*1 + 1*2 + A*3 + 1*4 + 1*5 + 5*6 + 6*7,
            ]
        );
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
