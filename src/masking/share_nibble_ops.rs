//! Nibble operations performed on shares.

use crate::masking::next_u112;
use crate::masking::nibble_ops::{nibble_mult_u112, nibble_square_u112};
use rand_chacha::ChaCha20Rng;

/// Raises the shares to the power of `2 ^ squares`.
/// # Panics
/// Will panic if `multiple_of_two` is zero.
#[inline]
pub(crate) fn share_nibble_square<const PX: u8, const SHARE_COUNT: usize>(
    shares: [u128; SHARE_COUNT],
) -> [u128; SHARE_COUNT] {
    shares.map(nibble_square_u112::<PX>)
}

pub(crate) fn share_nibble_mult<const PX: u8, const SHARE_COUNT: usize>(
    a: [u128; SHARE_COUNT],
    b: [u128; SHARE_COUNT],
    rng: &mut ChaCha20Rng,
) -> [u128; SHARE_COUNT] {
    let mut rng_table: [[u128; SHARE_COUNT]; SHARE_COUNT] =
        core::array::from_fn(|_| core::array::from_fn(|_| u128::default()));
    let mut result = core::array::from_fn(|_| u128::default());

    for i in 0..SHARE_COUNT {
        for j in (i + 1)..SHARE_COUNT {
            rng_table[i][j] = next_u112(rng);

            let ai_bj = nibble_mult_u112::<PX>(a[i], b[j]);
            let aj_bi = nibble_mult_u112::<PX>(a[j], b[i]);

            rng_table[j][i] = (rng_table[i][j] ^ ai_bj) ^ aj_bi;
        }
    }

    for i in 0..SHARE_COUNT {
        result[i] = nibble_mult_u112::<PX>(a[i], b[i]);

        for j in 0..SHARE_COUNT {
            if i != j {
                result[i] ^= rng_table[i][j];
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge_u128, share_add_u112, split_u112, split_u128};
    use rand_chacha::rand_core::SeedableRng;

    const PX: u8 = crate::masking::nibble_ops::PICARO_S_BOX;

    #[test]
    fn linear_addition() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..255 {
            for b in 0..255 {
                let shares = share_add_u112::<2>(split_u128(a, &mut rng), split_u128(b, &mut rng));

                assert_eq!(merge_u128(shares), a ^ b);
            }
        }
    }

    #[test]
    fn linear_squaring() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_nibble_square::<PX, 2>(split_u112(i, &mut rng));

            assert_eq!(merge_u128(shares), nibble_square_u112::<PX>(i));
        }
    }

    #[test]
    fn linear_multiplication() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..256 {
            for b in 0..256 {
                let shares = share_nibble_mult::<PX, 2>(
                    split_u112(a, &mut rng),
                    split_u112(b, &mut rng),
                    &mut rng,
                );

                assert_eq!(merge_u128(shares), nibble_mult_u112::<PX>(a, b));
            }
        }
    }
}
