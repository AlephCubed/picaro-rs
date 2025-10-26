#![cfg(feature = "_masking")]

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

#[cfg(feature = "masking_level_1")]
const MASKING_LEVEL: usize = 1;
#[cfg(feature = "masking_level_2")]
const MASKING_LEVEL: usize = 2;
#[cfg(feature = "masking_level_3")]
const MASKING_LEVEL: usize = 1;

// Todo Don't generate new RNG every time.
/// Splits a sensitive value into multiple shares, depending on the masking level.
fn split(secret: u128) -> [u128; MASKING_LEVEL + 1] {
    let mut rand = ChaCha20Rng::from_os_rng();
    let mut result: [u128; MASKING_LEVEL + 1] = Default::default();
    result[0] = secret;

    for i in 0..MASKING_LEVEL {
        let r = next_u128(&mut rand);
        result[i + 1] = r;
        result[0] ^= r;
    }

    result
}

pub fn next_u128(rng: &mut ChaCha20Rng) -> u128 {
    let a = rng.next_u64() as u128;
    let b = rng.next_u64() as u128;
    (a << 64) | b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Picaro;

    #[test]
    fn encrypt_decrypt() {
        let plaintext = 314159;

        let k = split(12345);
        let p = split(plaintext);

        let ciphertext = k
            .iter()
            .zip(p)
            .map(|(k, p)| {
                let cipher = Picaro::new(*k);
                cipher.encrypt(p)
            })
            .reduce(|a, b| a ^ b)
            .unwrap();

        assert_ne!(plaintext, ciphertext);

        let c = split(ciphertext);

        let result = k
            .iter()
            .zip(c)
            .map(|(k, c)| {
                let cipher = Picaro::new(*k);
                cipher.encrypt(c)
            })
            .reduce(|a, b| a ^ b)
            .unwrap();

        assert_eq!(plaintext, result);
    }
}
