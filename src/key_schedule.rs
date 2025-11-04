/// Creates a 112-bit round key from the main key.
pub(crate) const fn round_key(round: u8, main_key: u128) -> u128 {
    (extended_key(round, main_key) << 16) >> 16
}

/// The amount to shift for a given round, starting from round 1.
const OMEGA: [u32; 11] = [1, 16, 17, 32, 33, 85, 86, 101, 102, 117, 118];

/// Creates a 128-bit round key from the main key.
/// # Panics
/// Will panic if the round number is greater or equal to 12.
#[inline]
const fn extended_key(round: u8, main_key: u128) -> u128 {
    assert!(round < 12, "Round number must be less than 12.");

    match round {
        0 => main_key,
        _ if round % 2 == 0 => t(main_key).rotate_right(OMEGA[(round - 1) as usize]),
        _ => main_key.rotate_right(OMEGA[(round - 1) as usize]),
    }
}

/// Calculates the function T from the given main key.
///
/// T is defined by splitting the main key into four 32-bit numbers, forming a vector of length four.
/// This vector is multiplied by an inverted four-by-four identity matrix.
#[inline]
const fn t(main_key: u128) -> u128 {
    let split = split_u128_to_u32(main_key);

    combine_u32_to_u128([
        split[1] ^ split[2] ^ split[3],
        split[0] ^ split[2] ^ split[3],
        split[0] ^ split[1] ^ split[3],
        split[0] ^ split[1] ^ split[2],
    ])
}

#[inline]
const fn split_u128_to_u32(x: u128) -> [u32; 4] {
    [
        (x >> 96) as u32,
        (x >> 64) as u32,
        (x >> 32) as u32,
        x as u32,
    ]
}

#[inline]
const fn combine_u32_to_u128(parts: [u32; 4]) -> u128 {
    ((parts[0] as u128) << 96)
        | ((parts[1] as u128) << 64)
        | ((parts[2] as u128) << 32)
        | (parts[3] as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_keys_for_first_10m_main_keys() {
        for i in 0..=10000000 {
            for round in 0..12 {
                round_key(round, i);
            }
        }
    }

    #[test]
    fn round_keys_for_last_10m_main_keys() {
        for i in (u128::MAX - 10000000)..=u128::MAX {
            for round in 0..12 {
                round_key(round, i);
            }
        }
    }

    #[test]
    #[should_panic(expected = "Round number must be less than 12.")]
    fn large_round_number() {
        round_key(12, 12345);
    }
}
