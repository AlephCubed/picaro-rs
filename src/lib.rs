mod compression;
mod expansion;
mod key_schedule;
mod s_box;

use crate::compression::compression;
use crate::expansion::expansion;
use crate::key_schedule::round_key;
use crate::s_box::s_box;
use std::mem::swap;

/// Returns true if the key is weak.
pub fn is_weak_key(main_key: u128) -> bool {
    matches!(
        main_key,
        0 // 0000...
        | u128::MAX // 1111... 
        | 0x55555555555555555555555555555555 // 0101... 
        | 0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA // 1010...
    )
}

pub struct Picaro {
    main_key: u128,
}

impl Picaro {
    /// Creates a new Picaro object using the given key.
    /// # Panics
    /// Will panic if the key is one of Picaro's four [weak keys](is_weak_key).
    pub fn new(main_key: u128) -> Self {
        assert!(!is_weak_key(main_key), "Weak key was provided!");

        Self { main_key }
    }

    pub fn encrypt(&self, data: u128) -> u128 {
        let [mut left, mut right] = split_u128_to_u64(data);

        for round in 0..12 {
            let mut state = expansion(right);
            state ^= round_key(round, self.main_key);
            state = s_box(state);
            left ^= compression(state);

            swap(&mut left, &mut right);
        }

        combine_u64_to_u128([left, right])
    }
}

#[inline]
fn split_u128_to_u64(x: u128) -> [u64; 2] {
    [(x >> 64) as u64, x as u64]
}

#[inline]
fn combine_u64_to_u128(parts: [u64; 2]) -> u128 {
    ((parts[0] as u128) << 64) | (parts[1] as u128)
}
