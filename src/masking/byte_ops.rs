//! Mathematical operation performed on bytes GF(2^8).

pub(crate) const AES_S_BOX: u16 = 0b1_0001_1011;
pub(crate) const PICARO_EC: u16 = 0b1_0001_1101;

/// Squares all the bytes individually in the given finite field.
#[inline]
pub(crate) fn byte_square_u128<const PX: u16>(share: u128) -> u128 {
    u128::from_be_bytes(share.to_be_bytes().map(|b| byte_square::<PX>(b)))
}

/// Squares a byte in the given finite field.
#[inline]
fn byte_square<const PX: u16>(byte: u8) -> u8 {
    let mut result = 0u16;

    // Square using the Frobenius automorphism,`(x + y)^p = x^p + y^p`.
    // `(x^n)^2 = x^2n`, which is the same as doubling the place value of a bit.
    for i in 0..8 {
        result |= ((byte & (1 << i)) as u16) << i;
    }

    byte_reduce::<PX>(result)
}

/// Squares all the bytes individually in the given finite field.
#[inline]
pub(crate) fn byte_mult_u128<const PX: u16>(a: u128, b: u128) -> u128 {
    let mut a = a.to_be_bytes();
    let b = b.to_be_bytes();

    for i in 0..16 {
        a[i] = byte_mult::<PX>(a[i], b[i]);
    }

    u128::from_be_bytes(a)
}

/// Multiplies two bytes in the given finite field.
#[inline]
pub(crate) fn byte_mult<const PX: u16>(a: u8, b: u8) -> u8 {
    let mut result = 0u16;

    for i in 0..8 {
        if (b >> i) & 1 == 1 {
            result ^= (a as u16) << i;
        }
    }

    byte_reduce::<PX>(result)
}

#[inline]
fn byte_reduce<const PX: u16>(mut byte: u16) -> u8 {
    // We only need to go up to 14 because the maximum value is `x^7` in an 8-bit number.
    for i in (8..=14).rev() {
        if (byte >> i) & 1 == 1 {
            byte ^= PX << (i - 8);
        }
    }

    byte as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_byte_mult() {
        assert_eq!(byte_mult::<AES_S_BOX>(0x57, 0x83), 0xC1);
        assert_eq!(byte_mult::<AES_S_BOX>(0x57, 0x13), 0xFE);
        assert_eq!(byte_mult::<AES_S_BOX>(0x57, 0x02), 0xAE);
        assert_eq!(byte_mult::<AES_S_BOX>(0x57, 0x04), 0x47);
    }
}
