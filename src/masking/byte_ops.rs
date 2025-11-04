//! Mathematical operation performed on bytes modulo the AES polynomial.

const AES_POLYNOMIAL: u16 = 0b1_0001_1011;

/// Squares all the bytes individually in the AES finite field.
#[inline]
pub(crate) fn byte_square_u128(share: u128) -> u128 {
    u128::from_be_bytes(share.to_be_bytes().map(|b| byte_square(b)))
}

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
pub(crate) fn byte_multiplication_u128(a: u128, b: u128) -> u128 {
    let mut a = a.to_be_bytes();
    let b = b.to_be_bytes();

    for i in 0..16 {
        a[i] = byte_multiplication(a[i], b[i]);
    }

    u128::from_be_bytes(a)
}

/// Multiplies two bytes in the AES finite field.
#[inline]
pub(crate) fn byte_multiplication(a: u8, b: u8) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_multiplication() {
        assert_eq!(byte_multiplication(0x57, 0x83), 0xC1);
        assert_eq!(byte_multiplication(0x57, 0x13), 0xFE);
        assert_eq!(byte_multiplication(0x57, 0x02), 0xAE);
        assert_eq!(byte_multiplication(0x57, 0x04), 0x47);
    }
}
