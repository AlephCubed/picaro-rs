#![no_std]

mod compression;
mod expansion;
mod key_schedule;
mod masking;
mod s_box;

use crate::compression::compression;
use crate::expansion::expansion;
use crate::key_schedule::round_key;
use crate::s_box::s_box;
use core::mem::swap;

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
            self.round(round, &mut left, &mut right);
            swap(&mut left, &mut right);
        }

        // We swap the order to undo the swap from the last iteration.
        combine_u64_to_u128([right, left])
    }

    pub fn decrypt(&self, data: u128) -> u128 {
        let [mut left, mut right] = split_u128_to_u64(data);

        for round in (0..12).rev() {
            self.round(round, &mut left, &mut right);
            swap(&mut left, &mut right);
        }

        // We swap the order to undo the swap from the last iteration.
        combine_u64_to_u128([right, left])
    }

    fn round(&self, round: u8, left: &mut u64, right: &mut u64) {
        let mut state = expansion(*right);
        state ^= round_key(round, self.main_key);
        state = s_box(state);
        let result = compression(state);
        *left ^= result;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt() {
        let cipher = Picaro::new(12345);
        let plaintext = 314159;
        let ciphertext = cipher.encrypt(plaintext);

        assert_ne!(plaintext, ciphertext);

        let result = cipher.decrypt(ciphertext);
        assert_eq!(plaintext, result);
    }

    #[test]
    fn encrypt_decrypt_first_10k_plaintext() {
        for i in 0..=10000 {
            let cipher = Picaro::new(12345);
            let ciphertext = cipher.encrypt(i);

            assert_ne!(i, ciphertext);

            let result = cipher.decrypt(ciphertext);
            assert_eq!(i, result);
        }
    }

    #[test]
    fn encrypt_decrypt_last_10k_plaintext() {
        for i in (u128::MAX - 10000)..=u128::MAX {
            let cipher = Picaro::new(12345);
            let ciphertext = cipher.encrypt(i);

            assert_ne!(i, ciphertext);

            let result = cipher.decrypt(ciphertext);
            assert_eq!(i, result);
        }
    }

    #[test]
    fn encrypt_decrypt_first_10k_keys() {
        // Skip first key since it is weak.
        for i in 1..=10000 {
            let cipher = Picaro::new(i);
            let plaintext = 314159;
            let ciphertext = cipher.encrypt(plaintext);

            assert_ne!(plaintext, ciphertext);

            let result = cipher.decrypt(ciphertext);
            assert_eq!(plaintext, result);
        }
    }

    #[test]
    fn encrypt_decrypt_last_10k_keys() {
        // Skip last key since it is weak.
        for i in (u128::MAX - 10000)..u128::MAX {
            let cipher = Picaro::new(i);
            let plaintext = 314159;
            let ciphertext = cipher.encrypt(plaintext);

            assert_ne!(plaintext, ciphertext);

            let result = cipher.decrypt(ciphertext);
            assert_eq!(plaintext, result);
        }
    }
}
