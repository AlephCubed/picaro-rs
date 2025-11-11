#![no_std]

pub mod aes;
pub mod compression;
pub mod expansion;
pub mod key_schedule;
pub mod masking;
pub mod s_box;

use crate::compression::share_compression;
use crate::expansion::share_expansion;
use crate::key_schedule::round_key;
use crate::masking::{merge_u128, share_add_u64, share_add_u112, split_u128};
use crate::s_box::s_box;
use core::mem::swap;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;

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

pub struct Picaro<const SHARE_COUNT: usize = 1> {
    main_key: u128,
    rng: ChaCha20Rng,
}

impl<const SHARE_COUNT: usize> Picaro<SHARE_COUNT> {
    /// Creates a new Picaro object using the given key.
    /// # Panics
    /// Will panic if the key is one of Picaro's four [weak keys](is_weak_key).
    pub fn new_from_seed(main_key: u128, seed: u64) -> Self {
        assert!(!is_weak_key(main_key), "Weak key was provided!");

        Self {
            main_key,
            rng: ChaCha20Rng::seed_from_u64(seed),
        }
    }

    pub fn encrypt(&mut self, data: u128) -> u128 {
        let shares = split_u128::<SHARE_COUNT>(data, &mut self.rng);
        let key_shares = split_u128::<SHARE_COUNT>(self.main_key, &mut self.rng);

        let (mut left, mut right) = shares_u112_to_u64(shares);

        for round in 0..12 {
            self.round(round, &key_shares, &mut left, &right);
            swap(&mut left, &mut right);
        }

        // We swap the order to undo the swap from the last iteration.
        merge_u128(shares_u64_to_u128(right, left))
    }

    // Exact same as encryption but with a reversed key schedule.
    pub fn decrypt(&mut self, data: u128) -> u128 {
        let shares = split_u128::<SHARE_COUNT>(data, &mut self.rng);
        let key_shares = split_u128::<SHARE_COUNT>(self.main_key, &mut self.rng);

        let (mut left, mut right) = shares_u112_to_u64(shares);

        for round in (0..12).rev() {
            self.round(round, &key_shares, &mut left, &right);
            swap(&mut left, &mut right);
        }

        // We swap the order to undo the swap from the last iteration.
        merge_u128(shares_u64_to_u128(right, left))
    }

    fn round(
        &mut self,
        round: u8,
        key_shares: &[u128; SHARE_COUNT],
        left: &mut [u64; SHARE_COUNT],
        right: &[u64; SHARE_COUNT],
    ) {
        let mut state = share_expansion(*right);

        let key_shares = key_shares.map(|s| round_key(round, s));
        state = share_add_u112(state, key_shares);

        state = s_box(state, &mut self.rng);

        let result = share_compression(state);

        *left = share_add_u64(*left, result);
    }
}

#[inline]
fn shares_u112_to_u64<const SHARE_COUNT: usize>(
    shares: [u128; SHARE_COUNT],
) -> ([u64; SHARE_COUNT], [u64; SHARE_COUNT]) {
    let mut left = [0; SHARE_COUNT];
    let mut right = [0; SHARE_COUNT];

    for i in 0..SHARE_COUNT {
        left[i] = (shares[i] >> 64) as u64;
        right[i] = shares[i] as u64;
    }

    (left, right)
}

#[inline]
fn shares_u64_to_u128<const SHARE_COUNT: usize>(
    left: [u64; SHARE_COUNT],
    right: [u64; SHARE_COUNT],
) -> [u128; SHARE_COUNT] {
    let mut result = [0; SHARE_COUNT];

    for i in 0..SHARE_COUNT {
        result[i] = ((left[i] as u128) << 64) | (right[i] as u128)
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::next_u128;

    /// Tests that:
    /// 1. The cipher text is different from the plaintext.
    /// 2. Decrypting the ciphertext results in the plaintext.
    ///
    /// Returns the cipher text.
    fn encrypt_decrypt<const SHARE_COUNT: usize>(plaintext: u128, key: u128) -> u128 {
        let mut cipher = Picaro::<SHARE_COUNT>::new_from_seed(key, 12345);
        let ciphertext = cipher.encrypt(plaintext);

        assert_ne!(
            plaintext, ciphertext,
            "Ciphertext is same as plaintext (shares={SHARE_COUNT}, plaintext={plaintext}, key={key})"
        );

        let result = cipher.decrypt(ciphertext);
        assert_eq!(
            plaintext, result,
            "Final plaintext doesn't match (shares={SHARE_COUNT}, plaintext={plaintext}, key={key})"
        );

        ciphertext
    }

    /// For masking levels `0..=3`, tests that:
    /// 1. The cipher text is different from the plaintext.
    /// 2. Decrypting the ciphertext results in the plaintext.
    /// 3. The ciphertext is the same for all masking levels.
    fn encrypt_decrypt_masked(plaintext: u128, key: u128) {
        let unmasked = encrypt_decrypt::<1>(plaintext, key);
        assert_eq!(unmasked, encrypt_decrypt::<2>(plaintext, key));
        assert_eq!(unmasked, encrypt_decrypt::<3>(plaintext, key));
        assert_eq!(unmasked, encrypt_decrypt::<4>(plaintext, key));
    }

    #[test]
    fn encrypt_decrypt_first_hundred_plaintext() {
        for i in 0..=100 {
            encrypt_decrypt_masked(i, 01234);
        }
    }

    #[test]
    fn encrypt_decrypt_last_hundred_plaintext() {
        for i in (u128::MAX - 100)..=u128::MAX {
            encrypt_decrypt_masked(i, 01234);
        }
    }

    #[test]
    fn encrypt_decrypt_first_hundred_keys() {
        // Skip first key since it is weak.
        for i in 1..=100 {
            encrypt_decrypt_masked(56789, i);
        }
    }

    #[test]
    fn encrypt_decrypt_last_hundred_keys() {
        // Skip last key since it is weak.
        for i in (u128::MAX - 100)..u128::MAX {
            encrypt_decrypt_masked(56789, i);
        }
    }

    #[test]
    fn encrypt_decrypt_random_hundred_plaintext() {
        let mut rng = ChaCha20Rng::seed_from_u64(01234);
        for _ in 0..100 {
            encrypt_decrypt_masked(next_u128(&mut rng), 56789);
        }
    }

    #[test]
    fn encrypt_decrypt_random_hundred_keys() {
        let mut rng = ChaCha20Rng::seed_from_u64(01234);
        for _ in 0..100 {
            encrypt_decrypt_masked(56789, next_u128(&mut rng));
        }
    }
}
