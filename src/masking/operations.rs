use crate::masking::{SHARE_COUNT, Shares, next_u128};
use rand_chacha::ChaCha20Rng;

fn share_multiplication(a: Shares, b: Shares, rng: &mut ChaCha20Rng) -> Shares {
    let mut r: [[u128; SHARE_COUNT]; SHARE_COUNT] = Default::default();
    let mut result = Shares::default();

    for i in 0..SHARE_COUNT {
        for j in (i + 1)..SHARE_COUNT {
            r[i][j] = next_u128(rng);
            r[j][i] = (r[i][j] ^ a[i] & b[j]) ^ a[j] & b[i];
        }
    }

    for i in 0..SHARE_COUNT {
        result[i] = a[i] & b[i];

        for j in 0..SHARE_COUNT {
            if i != j {
                result[i] ^= r[i][j];
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masking::{merge, split};
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn multiplication() {
        let mut rng = ChaCha20Rng::from_os_rng();

        let a = 15;
        let b = 45;

        let shares = share_multiplication(split(a, &mut rng), split(b, &mut rng), &mut rng);

        assert_eq!(merge(shares), a & b);
    }
}
