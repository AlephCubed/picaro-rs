use crate::masking::{next_u128, Shares, SHARE_COUNT};
use rand_chacha::ChaCha20Rng;

/// Raises the shares to the power of `2 ^ multiple_of_two`.
/// # Panics
/// Will panic if `multiple_of_two` is zero
#[inline]
fn share_square(mut shares: Shares, multiple_of_two: u32) -> Shares {
    assert_ne!(multiple_of_two, 0);

    for i in 0..SHARE_COUNT {
        shares[i] = shares[i].wrapping_pow(2u32.pow(multiple_of_two));
    }

    shares
}

fn share_multiplication(a: Shares, b: Shares, rng: &mut ChaCha20Rng) -> Shares {
    let mut rng_table: [Shares; SHARE_COUNT] = Default::default();
    let mut result = Shares::default();

    for i in 0..SHARE_COUNT {
        for j in (i + 1)..SHARE_COUNT {
            rng_table[i][j] = next_u128(rng);
            rng_table[j][i] = (rng_table[i][j] ^ (a[i] & b[j])) ^ (a[j] & b[i]);
        }
    }

    for i in 0..SHARE_COUNT {
        result[i] = a[i] & b[i];

        for j in 0..SHARE_COUNT {
            if i != j {
                result[i] ^= rng_table[i][j];
            }
        }
    }

    result
}

fn share_pow(mut x: Shares, rng: &mut ChaCha20Rng) -> Shares {
    let z = share_square(x, 1);
    x = share_multiplication(z, x, rng);
    let w = share_square(x, 2);
    x = share_multiplication(x, w, rng);
    x = share_square(x, 4);
    x = share_multiplication(x, w, rng);
    share_multiplication(x, z, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge, split};
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn squaring() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 5;

        let shares = share_square(split(a, &mut rng), 1);

        assert_eq!(merge(shares), a.wrapping_pow(2));
    }

    #[test]
    fn double_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 5;

        let shares = share_square(split(a, &mut rng), 2);

        assert_eq!(merge(shares), a.wrapping_pow(4));
    }

    #[test]
    fn quadruple_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 5;

        let shares = share_square(split(a, &mut rng), 4);

        assert_eq!(merge(shares), a.wrapping_pow(16));
    }

    #[test]
    fn multiplication() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 15;
        let b = 45;

        let shares = share_multiplication(split(a, &mut rng), split(b, &mut rng), &mut rng);

        assert_eq!(merge(shares), a & b);
    }
}
