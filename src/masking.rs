#![cfg(feature = "_masking")]

mod operations;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::RngCore;

#[cfg(feature = "masking_level_1")]
const MASKING_LEVEL: usize = 1;
#[cfg(feature = "masking_level_2")]
const MASKING_LEVEL: usize = 2;
#[cfg(feature = "masking_level_3")]
const MASKING_LEVEL: usize = 1;

const SHARE_COUNT: usize = MASKING_LEVEL + 1;

type Shares = [u128; SHARE_COUNT];

// Todo Don't generate new RNG every time.
/// Splits a sensitive value into multiple shares, depending on the masking level.
#[inline]
fn split(secret: u128, rng: &mut ChaCha20Rng) -> Shares {
    let mut result = Shares::default();
    result[0] = secret;

    for i in 0..MASKING_LEVEL {
        let r = next_u128(rng);
        result[i + 1] = r;
        result[0] ^= r;
    }

    result
}

#[inline]
fn merge(shares: Shares) -> u128 {
    let mut result = shares[0];

    for i in 1..SHARE_COUNT {
        result ^= shares[i];
    }

    result
}

#[inline]
pub fn next_u128(rng: &mut ChaCha20Rng) -> u128 {
    let a = rng.next_u64() as u128;
    let b = rng.next_u64() as u128;
    (a << 64) | b
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn split_merge() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let secret = 12345;

        assert_eq!(merge(split(secret, &mut rng)), secret);
    }
}
