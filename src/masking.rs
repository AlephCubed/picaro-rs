pub mod byte_ops;
pub mod nibble_ops;
pub mod share_ops;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::RngCore;

#[inline]
pub(crate) fn split_u128<const SHARE_COUNT: usize>(
    secret: u128,
    rng: &mut ChaCha20Rng,
) -> [u128; SHARE_COUNT] {
    let mut result = core::array::from_fn(|_| u128::default());
    result[0] = secret;

    for i in 1..SHARE_COUNT {
        let r = next_u128(rng);
        result[i] = r;
        result[0] ^= r;
    }

    result
}

#[inline]
pub(crate) fn split_u64<const SHARE_COUNT: usize>(
    secret: u64,
    rng: &mut ChaCha20Rng,
) -> [u64; SHARE_COUNT] {
    let mut result = core::array::from_fn(|_| u64::default());
    result[0] = secret;

    for i in 1..SHARE_COUNT {
        let r = rng.next_u64();
        result[i] = r;
        result[0] ^= r;
    }

    result
}

#[inline]
pub(crate) fn refresh_masks<const SHARE_COUNT: usize>(
    shares: &mut [u128; SHARE_COUNT],
    rng: &mut ChaCha20Rng,
) {
    for i in 1..SHARE_COUNT {
        let temp = next_u128(rng);
        shares[0] ^= temp;
        shares[i] ^= temp;
    }
}

#[inline]
pub(crate) fn merge_u128<const SHARE_COUNT: usize>(shares: [u128; SHARE_COUNT]) -> u128 {
    let mut result = shares[0];

    for i in 1..SHARE_COUNT {
        result ^= shares[i];
    }

    result
}

#[inline]
pub(crate) fn merge_u64<const SHARE_COUNT: usize>(shares: [u64; SHARE_COUNT]) -> u64 {
    let mut result = shares[0];

    for i in 1..SHARE_COUNT {
        result ^= shares[i];
    }

    result
}

#[inline]
pub(crate) fn next_u128(rng: &mut ChaCha20Rng) -> u128 {
    let a = rng.next_u64() as u128;
    let b = rng.next_u64() as u128;
    (a << 64) | b
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn split_merge_u128() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let secret = 12345;

        assert_eq!(merge_u128::<1>(split_u128::<1>(secret, &mut rng)), secret);
        assert_eq!(merge_u128::<2>(split_u128::<2>(secret, &mut rng)), secret);
        assert_eq!(merge_u128::<3>(split_u128::<3>(secret, &mut rng)), secret);
        assert_eq!(merge_u128::<4>(split_u128::<4>(secret, &mut rng)), secret);
    }

    #[test]
    fn split_merge_u64() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let secret = 12345;

        assert_eq!(merge_u64::<1>(split_u64::<1>(secret, &mut rng)), secret);
        assert_eq!(merge_u64::<2>(split_u64::<2>(secret, &mut rng)), secret);
        assert_eq!(merge_u64::<3>(split_u64::<3>(secret, &mut rng)), secret);
        assert_eq!(merge_u64::<4>(split_u64::<4>(secret, &mut rng)), secret);
    }
}
