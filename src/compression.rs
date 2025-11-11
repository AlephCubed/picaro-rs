use crate::masking::byte_ops::{PICARO_EC, byte_mult};

/// The last 6 columns of the matrix G.
pub(crate) const G_LAST_SIX: [[u8; 6]; 8] = [
    [0x01, 0x01, 0x0A, 0x01, 0x09, 0x0C],
    [0x05, 0x01, 0x01, 0x0A, 0x01, 0x09],
    [0x06, 0x05, 0x01, 0x01, 0x0A, 0x01],
    [0x0C, 0x06, 0x05, 0x01, 0x01, 0x0A],
    [0x09, 0x0C, 0x06, 0x05, 0x01, 0x01],
    [0x01, 0x09, 0x0C, 0x06, 0x05, 0x01],
    [0x0A, 0x01, 0x09, 0x0C, 0x06, 0x05],
    [0x01, 0x0A, 0x01, 0x09, 0x0C, 0x06],
];

pub fn share_compression<const SHARE_COUNT: usize>(
    shares: [u128; SHARE_COUNT],
) -> [u64; SHARE_COUNT] {
    let mut result = [0; SHARE_COUNT];

    for i in 0..SHARE_COUNT {
        result[i] = compression(shares[i]);
    }

    result
}

/// Compresses the state into a 64-bit number.
fn compression(state: u128) -> u64 {
    debug_assert_eq!(state >> 112, 0, "Must be an 112-bit number.");

    let mut result = state as u64;

    for i in 0..8 {
        let mut sum = 0;

        for j in 0..6 {
            sum ^= byte_mult::<PICARO_EC>((state >> 8 * (8 + j)) as u8, G_LAST_SIX[i][j])
        }

        result ^= (sum as u64) << 8 * i;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge_u64, split_u128};
    use rand_chacha::ChaCha20Rng;
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    #[should_panic(expected = "Must be an 112-bit number.")]
    fn over_112_bits() {
        compression(u128::MAX);
    }

    #[test]
    fn test_compression_identity_first_100k() {
        for i in 0..100000 {
            assert_eq!(compression(i as u128), i);
        }
    }

    #[test]
    fn test_compression_identity_last_100k() {
        for i in (u64::MAX - 100000)..u64::MAX {
            assert_eq!(compression(i as u128), i);
        }
    }

    #[test]
    fn test_expansion_bytes_ones() {
        let input = 0x010101010101 << 8 * 8;

        let output = compression(input);

        for i in 0..6 {
            let sum = G_LAST_SIX[i].iter().fold(0, |sum, e| sum ^ *e);

            assert_eq!((output >> 8 * i) as u8, sum);
        }
    }

    fn test_masked_compression<const SHARE_COUNT: usize>(
        input: u128,
        rng: &mut ChaCha20Rng,
    ) -> u64 {
        let mut shares = split_u128::<SHARE_COUNT>(input << 16, rng);

        for i in 0..SHARE_COUNT {
            shares[i] >>= 16;
        }

        let compressed = share_compression(shares);
        merge_u64(compressed)
    }

    #[test]
    fn masked() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..256 {
            let unmasked = test_masked_compression::<1>(i, &mut rng);
            assert_eq!(unmasked, test_masked_compression::<2>(i, &mut rng));
            assert_eq!(unmasked, test_masked_compression::<3>(i, &mut rng));
            assert_eq!(unmasked, test_masked_compression::<4>(i, &mut rng));
        }
    }
}
