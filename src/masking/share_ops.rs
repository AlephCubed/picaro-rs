//! Mathematical operations performed on shares.

use crate::masking::byte_ops::{byte_multiplication_u128, byte_square_u128};
use crate::masking::{SHARE_COUNT, Shares, next_u128};
use rand_chacha::ChaCha20Rng;

#[inline]
fn share_addition(mut a: Shares, b: Shares) -> Shares {
    for i in 0..SHARE_COUNT {
        a[i] ^= b[i];
    }

    a
}

/// Raises the shares to the power of `2 ^ squares`.
/// # Panics
/// Will panic if `multiple_of_two` is zero.
#[inline]
pub(crate) fn share_square(mut shares: Shares, squares: u32) -> Shares {
    assert_ne!(squares, 0);

    for i in 0..SHARE_COUNT {
        for _ in 0..squares {
            shares[i] = byte_square_u128(shares[i]);
        }
    }

    shares
}

pub(crate) fn share_multiplication(a: Shares, b: Shares, rng: &mut ChaCha20Rng) -> Shares {
    let mut rng_table: [Shares; SHARE_COUNT] = Default::default();
    let mut result = Shares::default();

    for i in 0..SHARE_COUNT {
        for j in (i + 1)..SHARE_COUNT {
            rng_table[i][j] = next_u128(rng);

            let ai_bj = byte_multiplication_u128(a[i], b[j]);
            let aj_bi = byte_multiplication_u128(a[j], b[i]);

            rng_table[j][i] = (rng_table[i][j] ^ ai_bj) ^ aj_bi;
        }
    }

    for i in 0..SHARE_COUNT {
        result[i] = byte_multiplication_u128(a[i], b[i]);

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

    #[test]
    fn linear_addition() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..255 {
            for b in 0..255 {
                let shares = share_addition(split(a, &mut rng), split(b, &mut rng));

                assert_eq!(merge(shares), a ^ b);
            }
        }
    }

    #[test]
    fn linear_squaring() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_square(split(i, &mut rng), 1);

            assert_eq!(merge(shares), byte_square_u128(i));
        }
    }

    #[test]
    fn linear_double_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_square(split(i, &mut rng), 2);

            assert_eq!(merge(shares), byte_square_u128(byte_square_u128(i)));
        }
    }

    #[test]
    fn linear_quadruple_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for i in 0..255 {
            let shares = share_square(split(i, &mut rng), 3);

            assert_eq!(
                merge(shares),
                byte_square_u128(byte_square_u128(byte_square_u128(i)))
            );
        }
    }

    #[test]
    fn linear_multiplication() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for a in 0..255 {
            for b in 0..255 {
                let shares = share_multiplication(split(a, &mut rng), split(b, &mut rng), &mut rng);

                assert_eq!(merge(shares), byte_multiplication_u128(a, b));
            }
        }
    }
}
