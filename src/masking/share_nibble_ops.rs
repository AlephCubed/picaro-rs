//! Nibble operations performed on shares.

use crate::masking::nibble_ops::{nibble_mult_byte, nibble_square_byte};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::impls::fill_bytes_via_next;

/// Raises the shares to the power of `2 ^ squares`.
/// # Panics
/// Will panic if `multiple_of_two` is zero.
#[inline]
pub(crate) fn share_nibble_square<const PX: u8, const SHARE_COUNT: usize>(
    shares: [u8; SHARE_COUNT],
) -> [u8; SHARE_COUNT] {
    shares.map(nibble_square_byte::<PX>)
}

pub(crate) fn share_nibble_mult<const PX: u8, const SHARE_COUNT: usize>(
    a: [u8; SHARE_COUNT],
    b: [u8; SHARE_COUNT],
    rng: &mut ChaCha20Rng,
) -> [u8; SHARE_COUNT] {
    let mut rng_table = [[0; SHARE_COUNT]; SHARE_COUNT];
    let mut result = [0; SHARE_COUNT];

    for i in 0..SHARE_COUNT {
        // Todo Could theoretically be optimized more, by generating for multiple rows at once.
        fill_bytes_via_next(rng, &mut rng_table[i][(i + 1)..SHARE_COUNT]);

        for j in (i + 1)..SHARE_COUNT {
            let ai_bj = nibble_mult_byte::<PX>(a[i], b[j]);
            let aj_bi = nibble_mult_byte::<PX>(a[j], b[i]);

            rng_table[j][i] = (rng_table[i][j] ^ ai_bj) ^ aj_bi;
        }
    }

    for i in 0..SHARE_COUNT {
        result[i] = nibble_mult_byte::<PX>(a[i], b[i]);

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
    use crate::masking::nibble_ops::{nibble_mult_byte, nibble_square_byte};
    use crate::masking::{merge_u8, share_add_u8, split_u8};
    use rand_chacha::rand_core::SeedableRng;

    const PX: u8 = crate::masking::nibble_ops::PICARO_S_BOX;

    #[test]
    fn linear_addition() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..u8::MAX {
            for b in 0..u8::MAX {
                let shares = share_add_u8::<2>(split_u8(a, &mut rng), split_u8(b, &mut rng));

                assert_eq!(merge_u8(shares), a ^ b);
            }
        }
    }

    #[test]
    fn linear_squaring() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..u8::MAX {
            let shares = share_nibble_square::<PX, 2>(split_u8(i, &mut rng));

            assert_eq!(merge_u8(shares), nibble_square_byte::<PX>(i));
        }
    }

    #[test]
    fn linear_multiplication() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..u8::MAX {
            for b in 0..u8::MAX {
                let shares = share_nibble_mult::<PX, 2>(
                    split_u8(a, &mut rng),
                    split_u8(b, &mut rng),
                    &mut rng,
                );

                assert_eq!(merge_u8(shares), nibble_mult_byte::<PX>(a, b));
            }
        }
    }
}
