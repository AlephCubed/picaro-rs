use crate::masking::byte_ops::{byte_mult, PICARO_EX};

/// The last 6 columns of the matrix G.
const G_LAST_SIX: [[u8; 6]; 8] = [
    [0x01, 0x01, 0x0A, 0x01, 0x09, 0x0C],
    [0x05, 0x01, 0x01, 0x0A, 0x01, 0x09],
    [0x06, 0x05, 0x01, 0x01, 0x0A, 0x01],
    [0x0C, 0x06, 0x05, 0x01, 0x01, 0x0A],
    [0x09, 0x0C, 0x06, 0x05, 0x01, 0x01],
    [0x01, 0x09, 0x0C, 0x06, 0x05, 0x01],
    [0x0A, 0x01, 0x09, 0x0C, 0x06, 0x05],
    [0x01, 0x0A, 0x01, 0x09, 0x0C, 0x06],
];

// Todo Performance might be bad.
/// Compresses the state into a 64-bit number.
pub(crate) fn compression(state: u128) -> u64 {
    assert_eq!(state >> 112, 0, "Must be an 112-bit number.");

    let bytes = state.to_be_bytes();
    let mut result = (state as u64).to_be_bytes();

    for i in 0..8 {
        result[i] ^= bytes
            .iter()
            .skip(8)
            .zip(G_LAST_SIX[i])
            .map(|(byte, g)| byte_mult::<PICARO_EX>(*byte, g))
            .reduce(|sum, e| sum ^ e)
            .expect("There will always be 6 elements");
    }

    u64::from_be_bytes(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let right = u128::from_be_bytes([
            0, 0, // First two bytes are skipped for 112-bit.
            0, 1, 2, 3, 4, 5, // The combination portion.
            0, 1, 2, 3, 4, 5, 6, 7, // The identity portion.
        ]);
        let result = compression(right);

        // Constants to align the expected grid.
        const A: u8 = 0xa;
        const C: u8 = 0xC;

        #[rustfmt::skip]
        assert_eq!(
            result.to_be_bytes(),
            [
                // First number is the identity, which is added to the linear combination.
                0 + 0*1 + 1*1 + 2*A + 3*1 + 4*9 + 5*C,
                1 + 0*5 + 1*1 + 2*1 + 3*A + 4*1 + 5*9,
                2 + 0*6 + 1*5 + 2*1 + 3*1 + 4*A + 5*1,
                3 + 0*C + 1*6 + 2*5 + 3*1 + 4*1 + 5*A,
                4 + 0*9 + 1*C + 2*6 + 3*5 + 4*1 + 5*1,
                5 + 0*1 + 1*9 + 2*C + 3*6 + 4*5 + 5*1,
                6 + 0*A + 1*1 + 2*9 + 3*C + 4*6 + 5*5,
                7 + 0*1 + 1*A + 2*1 + 3*9 + 4*C + 5*6,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "Must be an 112-bit number.")]
    fn over_112_bits() {
        compression(u128::MAX);
    }
}
