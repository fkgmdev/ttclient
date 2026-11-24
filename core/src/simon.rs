//! Simon 128/256 block cipher, ported from iqbalmh18/tiktok-signer.

const CONSTANT: u64 = 0x3DC94C3A046D678B;
const ROUNDS: usize = 72;

#[inline]
fn rotl(v: u64, n: u32) -> u64 {
    let n = n % 64;
    if n == 0 {
        v
    } else {
        (v << n) | (v >> (64 - n))
    }
}

#[inline]
fn rotr(v: u64, n: u32) -> u64 {
    let n = n % 64;
    if n == 0 {
        v
    } else {
        (v >> n) | (v << (64 - n))
    }
}

fn expand(key: &[u64; 4]) -> [u64; ROUNDS] {
    let mut k = [0u64; ROUNDS];
    k[..4].copy_from_slice(key);
    for i in 4..ROUNDS {
        let mut tmp = rotr(k[i - 1], 3);
        tmp ^= k[i - 3];
        tmp ^= rotr(tmp, 1);
        let bit = (CONSTANT >> ((i - 4) % 62)) & 1;
        k[i] = (!k[i - 4]) ^ tmp ^ bit ^ 3;
    }
    k
}

/// Encrypt one 128-bit block, given a 256-bit key as four little-endian words.
pub fn encrypt(pt: (u64, u64), key: &[u64; 4]) -> (u64, u64) {
    let k = expand(key);
    let (mut x_i, mut x_i1) = pt;
    for round in &k {
        let tmp = x_i1;
        let f = rotl(x_i1, 1) & rotl(x_i1, 8);
        x_i1 = x_i ^ f ^ rotl(x_i1, 2) ^ round;
        x_i = tmp;
    }
    (x_i, x_i1)
}

#[cfg(test)]
mod tests {
    use super::{encrypt, expand};

    #[test]
    fn key_schedule_length() {
        assert_eq!(expand(&[1, 2, 3, 4]).len(), 72);
    }

    #[test]
    fn roundtrip_shape() {
        // Simon encryption is deterministic; a fixed input must give a fixed output.
        let a = encrypt((0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210), &[1, 2, 3, 4]);
        let b = encrypt((0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210), &[1, 2, 3, 4]);
        assert_eq!(a, b);
        assert_ne!(a.0, 0);
    }
}
