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

const AES_POLYNOMIAL: u16 = 0x1B;

/// Squares a byte in the AES finite field.
#[inline]
fn byte_square(byte: u8) -> u8 {
    let mut result = 0u16;

    // Square using the Frobenius automorphism,`(x + y)^p = x^p + y^p`.
    // `(x^n)^2 = x^2n`, which is the same as doubling the place value of a bit.
    for i in 0..8 {
        result |= ((byte & (1 << i)) as u16) << i;
    }

    byte_reduce(result)
}

/// Squares all the bytes individually in the AES finite field.
#[inline]
fn byte_multiplication_u128(a: u128, b: u128) -> u128 {
    let mut a = a.to_be_bytes();
    let b = b.to_be_bytes();

    for i in 0..8 {
        a[i] = byte_multiplication(a[i], b[i]);
    }

    u128::from_be_bytes(a)
}

/// Multiplies two bytes in the AES finite field.
fn byte_multiplication(a: u8, b: u8) -> u8 {
    let mut result = 0u16;

    for i in 0..8 {
        if (b >> i) & 1 == 1 {
            result ^= (a as u16) << i;
        }
    }

    byte_reduce(result)
}

/// Reduce using `x^8 = x^4 + x^3 + x + 1`.
/// We only need to go up to 14 because the maximum value is `x^7` in an 8-bit number.
#[inline]
fn byte_reduce(mut byte: u16) -> u8 {
    for i in (8..=14).rev() {
        if (byte >> i) & 1 == 1 {
            byte ^= AES_POLYNOMIAL << (i - 8);
        }
    }

    byte as u8
}

fn share_multiplication(a: Shares, b: Shares, rng: &mut ChaCha20Rng) -> Shares {
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
    let mut w = share_square(y, 2); // XOR of w = x^12
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

fn aes_affine_transformation(mut shares: Shares) -> Shares {
    for i in 0..SHARE_COUNT {
        let mut bytes = shares[i].to_be_bytes();

        for j in 0..16 {
            bytes[j] = bytes[j]
                ^ bytes[j].rotate_left(1)
                ^ bytes[j].rotate_left(2)
                ^ bytes[j].rotate_left(3)
                ^ bytes[j].rotate_left(4);
        }

        shares[i] = u128::from_be_bytes(bytes) ^ 0x63;
    }

    shares
}

fn aes_s_box(mut shares: Shares, rng: &mut ChaCha20Rng) -> Shares {
    shares = share_pow_254(shares, rng);
    shares = aes_affine_transformation(shares);

    if SHARE_COUNT % 2 == 1 {
        shares[0] ^= 0x63;
    }

    shares
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

    #[test]
    fn inversion() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let first_row: [u8; 16] = [
            0x00, 0x01, 0x8D, 0xF6, 0xCB, 0x52, 0x7B, 0xD1, 0xE8, 0x4F, 0x29, 0xC0, 0xB0, 0xE1,
            0xE5, 0xC7,
        ];

        for i in 0..16 {
            let result = share_pow_254(split(i, &mut rng), &mut rng);
            assert_eq!(merge(result).to_be_bytes()[15], first_row[i as usize]);
        }
    }

    #[test]
    fn aes() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let first_row: [u8; 16] = [
            0x63, 0x7C, 0x77, 0x7B, 0xF2, 0x6B, 0x6F, 0xC5, 0x30, 0x01, 0x67, 0x2B, 0xFE, 0xD7,
            0xAB, 0x76,
        ];

        for i in 0..16 {
            let result = aes_s_box(split(i, &mut rng), &mut rng);
            assert_eq!(merge(result).to_be_bytes()[15], first_row[i as usize]);
        }
    }
}
