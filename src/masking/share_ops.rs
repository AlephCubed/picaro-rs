//! Mathematical operations performed on shares.

use crate::masking::byte_ops::{byte_mult_u128, byte_square_u128};
use crate::masking::next_u128;
use rand_chacha::ChaCha20Rng;

#[inline]
fn share_addition<const SHARE_COUNT: usize>(
    mut a: [u128; SHARE_COUNT],
    b: [u128; SHARE_COUNT],
) -> [u128; SHARE_COUNT] {
    for i in 0..SHARE_COUNT {
        a[i] ^= b[i];
    }

    a
}

/// Raises the shares to the power of `2 ^ squares`.
/// # Panics
/// Will panic if `multiple_of_two` is zero.
#[inline]
pub(crate) fn share_square<const PX: u16, const SHARE_COUNT: usize>(
    mut shares: [u128; SHARE_COUNT],
    squares: u32,
) -> [u128; SHARE_COUNT] {
    assert_ne!(squares, 0);

    for i in 0..SHARE_COUNT {
        for _ in 0..squares {
            shares[i] = byte_square_u128::<PX>(shares[i]);
        }
    }

    shares
}

pub(crate) fn share_mult<const PX: u16, const SHARE_COUNT: usize>(
    a: [u128; SHARE_COUNT],
    b: [u128; SHARE_COUNT],
    rng: &mut ChaCha20Rng,
) -> [u128; SHARE_COUNT] {
    let mut rng_table: [[u128; SHARE_COUNT]; SHARE_COUNT] =
        core::array::from_fn(|_| core::array::from_fn(|_| u128::default()));
    let mut result = core::array::from_fn(|_| u128::default());

    for i in 0..SHARE_COUNT {
        for j in (i + 1)..SHARE_COUNT {
            rng_table[i][j] = next_u128(rng);

            let ai_bj = byte_mult_u128::<PX>(a[i], b[j]);
            let aj_bi = byte_mult_u128::<PX>(a[j], b[i]);

            rng_table[j][i] = (rng_table[i][j] ^ ai_bj) ^ aj_bi;
        }
    }

    for i in 0..SHARE_COUNT {
        result[i] = byte_mult_u128::<PX>(a[i], b[i]);

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
    use crate::masking::{merge, split};
    use rand_chacha::rand_core::SeedableRng;

    const PX: u16 = crate::masking::byte_ops::AES_S_BOX;

    #[test]
    fn linear_addition() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..255 {
            for b in 0..255 {
                let shares = share_addition::<2>(split(a, &mut rng), split(b, &mut rng));

                assert_eq!(merge(shares), a ^ b);
            }
        }
    }

    #[test]
    fn linear_squaring() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_square::<PX, 2>(split(i, &mut rng), 1);

            assert_eq!(merge(shares), byte_square_u128::<PX>(i));
        }
    }

    #[test]
    fn linear_double_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_square::<PX, 2>(split(i, &mut rng), 2);

            assert_eq!(
                merge(shares),
                byte_square_u128::<PX>(byte_square_u128::<PX>(i))
            );
        }
    }

    #[test]
    fn linear_quadruple_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_square::<PX, 2>(split(i, &mut rng), 3);

            assert_eq!(
                merge(shares),
                byte_square_u128::<PX>(byte_square_u128::<PX>(byte_square_u128::<PX>(i)))
            );
        }
    }

    #[test]
    fn linear_multiplication() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..255 {
            for b in 0..255 {
                let shares = share_mult::<PX, 2>(split(a, &mut rng), split(b, &mut rng), &mut rng);

                assert_eq!(merge(shares), byte_mult_u128::<PX>(a, b));
            }
        }
    }
}
