use crate::masking::{next_u128, Shares, SHARE_COUNT};
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
fn share_square(mut shares: Shares, squares: u32) -> Shares {
    assert_ne!(squares, 0);

    for i in 0..SHARE_COUNT {
        for _ in 0..squares {
            shares[i] = byte_square_u128(shares[i]);
        }
    }

    shares
}

/// Squares all the bytes individually in the AES finite field.
#[inline]
fn byte_square_u128(share: u128) -> u128 {
    u128::from_be_bytes(share.to_be_bytes().map(|b| byte_square(b)))
}

const AES_POLYNOMIAL: u16 = 0b100011011;

/// Squares a byte in the AES finite field.
#[inline]
fn byte_square(byte: u8) -> u8 {
    let mut result = 0u16;

    // Square using the Frobenius automorphism,`(x + y)^p = x^p + y^p`.
    // `(x^n)^2 = x^2n`, which is the same as doubling the place value of a bit.
    for i in 0..8 {
        result |= ((byte & (1 << i)) as u16) << i;
    }

    // Reduce using `x^8 = x^4 + x^3 + x + 1`.
    // We only need to go up to 14 because the maximum value is `x^7` in an 8-bit number.
    for i in (8..=14).rev() {
        if result & (1 << i) == 1 {
            result ^= AES_POLYNOMIAL << (i - 8);
        }
    }

    result as u8
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

/// Used in AES:
fn share_pow_254(x: Shares, rng: &mut ChaCha20Rng) -> Shares {
    let mut z = share_square(x, 1); // XOR of z = x^2
    refresh_masks(&mut z, rng);
    let mut y = share_multiplication(z, x, rng); // XOR of y = x^3
    let mut w = share_square(x, 2); // XOR of w = x^12
    refresh_masks(&mut w, rng);
    y = share_multiplication(y, w, rng); // XOR of y = x^15
    y = share_square(y, 4); // XOR of y = x^240
    y = share_multiplication(y, w, rng); // XOR of y = x^252
    share_multiplication(y, z, rng) // XOR of y = x^254
}

#[inline]
fn refresh_masks(shares: &mut Shares, rng: &mut ChaCha20Rng) {
    for i in 1..SHARE_COUNT {
        let temp = next_u128(rng);
        shares[0] ^= temp;
        shares[i] ^= temp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge, split};
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn linear_addition() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 15;
        let b = 45;

        let shares = share_addition(split(a, &mut rng), split(b, &mut rng));

        assert_eq!(merge(shares), a ^ b);
    }

    #[test]
    fn linear_squaring() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 5;

        let shares = share_square(split(a, &mut rng), 1);

        assert_eq!(merge(shares), byte_square_u128(a));
    }

    #[test]
    fn linear_double_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 5;

        let shares = share_square(split(a, &mut rng), 2);

        assert_eq!(merge(shares), byte_square_u128(byte_square_u128(a)));
    }

    #[test]
    fn linear_quadruple_square() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 5;

        let shares = share_square(split(a, &mut rng), 3);

        assert_eq!(
            merge(shares),
            byte_square_u128(byte_square_u128(byte_square_u128(a)))
        );
    }

    #[test]
    fn linear_multiplication() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let a = 15;
        let b = 45;

        let shares = share_multiplication(split(a, &mut rng), split(b, &mut rng), &mut rng);

        assert_eq!(merge(shares), a & b);
    }
}
