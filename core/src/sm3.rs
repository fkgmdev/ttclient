//! SM3 hash (GB/T 32905-2016), ported from the MIT-licensed
//! iqbalmh18/tiktok-signer implementation.

const IV: [u32; 8] = [
    1937774191, 1226093241, 388252375, 3666478592, 2842636476, 372324522, 3817729613, 2969243214,
];

const TJ: [u32; 64] = [
    2043430169, 2043430169, 2043430169, 2043430169, 2043430169, 2043430169, 2043430169, 2043430169,
    2043430169, 2043430169, 2043430169, 2043430169, 2043430169, 2043430169, 2043430169, 2043430169,
    2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042,
    2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042,
    2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042,
    2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042,
    2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042,
    2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042, 2055708042,
];

#[inline]
fn rotl(a: u32, k: u32) -> u32 {
    let k = k % 32;
    if k == 0 {
        a
    } else {
        (a << k) | (a >> (32 - k))
    }
}

#[inline]
fn p0(x: u32) -> u32 {
    x ^ rotl(x, 9) ^ rotl(x, 17)
}

#[inline]
fn p1(x: u32) -> u32 {
    x ^ rotl(x, 15) ^ rotl(x, 23)
}

fn cf(v: [u32; 8], b: &[u8]) -> [u32; 8] {
    let mut w = [0u32; 68];
    for i in 0..16 {
        let mut data = 0u32;
        for k in 0..4 {
            data += (b[i * 4 + k] as u32) << (24 - 8 * k);
        }
        w[i] = data;
    }
    for j in 16..68 {
        w[j] = p1(w[j - 16] ^ w[j - 9] ^ rotl(w[j - 3], 15)) ^ rotl(w[j - 13], 7) ^ w[j - 6];
    }
    let mut w1 = [0u32; 64];
    for j in 0..64 {
        w1[j] = w[j] ^ w[j + 4];
    }

    let (mut a, mut b0, mut c, mut d, mut e, mut f, mut g, mut h) =
        (v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);

    for j in 0..64 {
        let ss1 = rotl(
            rotl(a, 12).wrapping_add(e).wrapping_add(rotl(TJ[j], j as u32)),
            7,
        );
        let ss2 = ss1 ^ rotl(a, 12);
        let ff = if j < 16 {
            a ^ b0 ^ c
        } else {
            (a & b0) | (a & c) | (b0 & c)
        };
        let gg = if j < 16 {
            e ^ f ^ g
        } else {
            (e & f) | ((!e) & g)
        };
        let tt1 = ff.wrapping_add(d).wrapping_add(ss2).wrapping_add(w1[j]);
        let tt2 = gg.wrapping_add(h).wrapping_add(ss1).wrapping_add(w[j]);
        d = c;
        c = rotl(b0, 9);
        b0 = a;
        a = tt1;
        h = g;
        g = rotl(f, 19);
        f = e;
        e = p0(tt2);
    }

    [
        a ^ v[0], b0 ^ v[1], c ^ v[2], d ^ v[3], e ^ v[4], f ^ v[5], g ^ v[6], h ^ v[7],
    ]
}

/// Compute the 32-byte SM3 digest of `msg`.
pub fn digest(msg: &[u8]) -> [u8; 32] {
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    let mut padded = msg.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut v = IV;
    for block in padded.chunks_exact(64) {
        v = cf(v, block);
    }
    let mut out = [0u8; 32];
    for (i, word) in v.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::digest;

    #[test]
    fn empty_vector() {
        let d = digest(b"");
        assert_eq!(
            d.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
            "1ab21d8355cfa17f8e61194831e81a8f22bec8c728fefb747ed035eb5082aa2b"
        );
    }

    #[test]
    fn abc_vector() {
        let d = digest(b"abc");
        assert_eq!(
            d.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
            "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"
        );
    }
}
