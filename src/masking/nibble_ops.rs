//! Mathematical operation performed on nibbles GF(2^4).

pub(crate) const PICARO_S_BOX: u8 = 0b11001;

#[macro_export]
macro_rules! nibble_op {
    ($byte:expr, $func:path) => {
        ($func($byte & 0b1111) << 4) ^ $func($byte >> 4)
    };
    ($a:expr, $b:expr, $func:path) => {
        ($func($a & 0b1111, $b & 0b1111) << 4) ^ $func($a >> 4, $b >> 4)
    };
}

/// Squares all the bytes individually in the given finite field.
#[inline]
pub(crate) fn nibble_square_u128<const PX: u8>(share: u128) -> u128 {
    u128::from_be_bytes(
        share
            .to_be_bytes()
            .map(|b| nibble_op!(b, nibble_square::<PX>)),
    )
}

/// Squares a nibble in the given finite field.
#[inline]
pub(crate) fn nibble_square<const PX: u8>(nibble: u8) -> u8 {
    debug_assert_eq!(nibble >> 4, 0, "Must be an 4-bit number.");

    let mut result = 0u8;

    // Square using the Frobenius automorphism,`(x + y)^p = x^p + y^p`.
    // `(x^n)^2 = x^2n`, which is the same as doubling the place value of a bit.
    for i in 0..4 {
        result |= (nibble & (1 << i)) << i;
    }

    nibble_reduce::<PX>(result)
}

/// Squares all the bytes individually in the given finite field.
#[inline]
pub(crate) fn nibble_mult_u128<const PX: u8>(a: u128, b: u128) -> u128 {
    let mut a = a.to_be_bytes();
    let b = b.to_be_bytes();

    for i in 0..16 {
        a[i] = nibble_op!(a[i], b[i], nibble_mult::<PX>);
    }

    u128::from_be_bytes(a)
}

/// Multiplies two nibble in the given field.
#[inline]
pub(crate) fn nibble_mult<const PX: u8>(a: u8, b: u8) -> u8 {
    debug_assert_eq!(a >> 4, 0, "`a` must be an 4-bit number.");
    debug_assert_eq!(b >> 4, 0, "`b` must be an 4-bit number.");

    let mut result = 0u8;

    for i in 0..4 {
        if (b >> i) & 1 == 1 {
            result ^= a << i;
        }
    }

    nibble_reduce::<PX>(result)
}

#[inline]
fn nibble_reduce<const PX: u8>(mut byte: u8) -> u8 {
    // We only need to go up to 6 because the maximum value is `x^3` in an 4-bit number.
    for i in (4..=6).rev() {
        if (byte >> i) & 1 == 1 {
            byte ^= PX << (i - 4);
        }
    }

    byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Must be an 4-bit number.")]
    fn invalid_input_square() {
        nibble_square::<PICARO_S_BOX>(0b10000);
    }

    #[test]
    #[should_panic(expected = "`a` must be an 4-bit number.")]
    fn invalid_input_mult_a() {
        nibble_mult::<PICARO_S_BOX>(0b10000, 0b1);
    }

    #[test]
    #[should_panic(expected = "`b` must be an 4-bit number.")]
    fn invalid_input_mult_b() {
        nibble_mult::<PICARO_S_BOX>(0b1, 0b10000);
    }
}
