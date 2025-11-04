//! A temporary holding place for AES masking.

use crate::masking::byte_ops::AES_S_BOX;
use crate::masking::refresh_masks;
use crate::masking::share_ops::{share_mult, share_square};
use rand_chacha::ChaCha20Rng;

/// Used in AES:
fn aes_inversion<const SHARE_COUNT: usize>(
    x: [u128; SHARE_COUNT],
    rng: &mut ChaCha20Rng,
) -> [u128; SHARE_COUNT] {
    let mut z = share_square::<AES_S_BOX, SHARE_COUNT>(x, 1); // XOR of z = x^2
    refresh_masks(&mut z, rng);
    let mut y = share_mult::<AES_S_BOX, SHARE_COUNT>(z, x, rng); // XOR of y = x^3
    let mut w = share_square::<AES_S_BOX, SHARE_COUNT>(y, 2); // XOR of w = x^12
    refresh_masks(&mut w, rng);
    y = share_mult::<AES_S_BOX, SHARE_COUNT>(y, w, rng); // XOR of y = x^15
    y = share_square::<AES_S_BOX, SHARE_COUNT>(y, 4); // XOR of y = x^240
    y = share_mult::<AES_S_BOX, SHARE_COUNT>(y, w, rng); // XOR of y = x^252
    share_mult::<AES_S_BOX, SHARE_COUNT>(y, z, rng) // XOR of y = x^254
}

fn aes_affine_transformation<const SHARE_COUNT: usize>(
    mut shares: [u128; SHARE_COUNT],
) -> [u128; SHARE_COUNT] {
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

fn aes_s_box<const SHARE_COUNT: usize>(
    mut shares: [u128; SHARE_COUNT],
    rng: &mut ChaCha20Rng,
) -> [u128; SHARE_COUNT] {
    shares = aes_inversion(shares, rng);
    shares = aes_affine_transformation(shares);

    if (SHARE_COUNT - 1) % 2 == 1 {
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
    fn test_aes_inversion() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let first_row: [u8; 16] = [
            0x00, 0x01, 0x8D, 0xF6, 0xCB, 0x52, 0x7B, 0xD1, 0xE8, 0x4F, 0x29, 0xC0, 0xB0, 0xE1,
            0xE5, 0xC7,
        ];

        for i in 0..16 {
            let result = aes_inversion::<2>(split(i, &mut rng), &mut rng);
            assert_eq!(merge(result).to_be_bytes()[15], first_row[i as usize]);
        }
    }

    #[test]
    fn test_aes_s_box() {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let first_row: [u8; 16] = [
            0x63, 0x7C, 0x77, 0x7B, 0xF2, 0x6B, 0x6F, 0xC5, 0x30, 0x01, 0x67, 0x2B, 0xFE, 0xD7,
            0xAB, 0x76,
        ];

        for i in 0..16 {
            let result = aes_s_box::<2>(split(i, &mut rng), &mut rng);
            assert_eq!(merge(result).to_be_bytes()[15], first_row[i as usize]);
        }
    }
}
