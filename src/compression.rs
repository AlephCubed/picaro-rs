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

pub(crate) fn share_compression<const SHARE_COUNT: usize>(
    shares: [u128; SHARE_COUNT],
) -> [u64; SHARE_COUNT] {
    let mut result = [0; SHARE_COUNT];

    for i in 0..SHARE_COUNT {
        result[i] = compression(shares[i]);
    }

    result
}

// Todo Performance might be bad.
/// Compresses the state into a 64-bit number.
pub(crate) fn compression(state: u128) -> u64 {
    debug_assert_eq!(state >> 112, 0, "Must be an 112-bit number.");

    let bytes = state.to_be_bytes();
    let mut result = (state as u64).to_be_bytes();

    for i in 0..8 {
        result[i] ^= bytes
            .iter()
            .skip(8)
            .zip(G_LAST_SIX[i])
            .map(|(byte, g)| byte_mult::<PICARO_EC>(*byte, g))
            .reduce(|sum, e| sum ^ e)
            .expect("There will always be 6 elements");
    }

    u64::from_be_bytes(result)
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
