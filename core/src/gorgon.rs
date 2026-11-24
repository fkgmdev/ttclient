//! X-Gorgon / X-Khronos generation, ported from the MIT-licensed
//! iqbalmh18/tiktok-signer implementation.

const LENGTH: usize = 20;
const HEX_STR: [u8; 8] = [30, 64, 224, 217, 147, 69, 0, 180];

fn md5_hex(data: &[u8]) -> String {
    use md5::Digest;
    hex::encode(md5::Md5::digest(data))
}

fn encryption_table() -> [u8; 256] {
    let mut hex_zu: Vec<u8> = (0..=255).collect();
    let mut last: Option<u8> = None;
    for i in 0..256 {
        let mut a = if i == 0 {
            0
        } else {
            match last {
                Some(v) if v != 0 => v,
                _ => hex_zu[i - 1],
            }
        };
        let b = HEX_STR[i % 8];
        if a == 85 && i != 1 && last != Some(85) {
            a = 0;
        }
        let c = (a as usize + i + b as usize) % 256;
        last = if c < i { Some(c as u8) } else { None };
        let d = hex_zu[c];
        hex_zu[i] = d;
    }
    let mut out = [0u8; 256];
    out.copy_from_slice(&hex_zu);
    out
}

fn initialize(input: &mut [u8; LENGTH], table: &[u8; 256]) {
    let mut tmp_add: Vec<u8> = Vec::with_capacity(LENGTH);
    let mut tmp_hex = *table;
    for i in 0..LENGTH {
        let a = input[i];
        let b = tmp_add.last().copied().unwrap_or(0);
        let c = (table[i + 1] as usize + b as usize) % 256;
        tmp_add.push(c as u8);
        let d = tmp_hex[c];
        tmp_hex[i + 1] = d;
        let e = (d as usize + d as usize) % 256;
        let f = tmp_hex[e];
        input[i] = a ^ f;
    }
}

#[inline]
fn reverse_nibbles(n: u8) -> u8 {
    (n << 4) | (n >> 4)
}

#[inline]
fn reverse_bits(mut n: u8) -> u8 {
    let mut out = 0u8;
    for _ in 0..8 {
        out = (out << 1) | (n & 1);
        n >>= 1;
    }
    out
}

fn handle(input: &mut [u8; LENGTH]) {
    for i in 0..LENGTH {
        let b = reverse_nibbles(input[i]);
        let c = input[(i + 1) % LENGTH];
        let d = b ^ c;
        let e = reverse_bits(d);
        let f = e ^ (LENGTH as u8);
        input[i] = !f;
    }
}

fn calculate(mut gorgon: [u8; LENGTH]) -> String {
    let table = encryption_table();
    initialize(&mut gorgon, &table);
    handle(&mut gorgon);
    let mut result = String::from("8404");
    for &n in &[HEX_STR[7], HEX_STR[3], HEX_STR[1], HEX_STR[6]] {
        result.push_str(&format!("{:02x}", n));
    }
    for byte in gorgon {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}

/// Generate the `x-gorgon` and `x-khronos` header values.
///
/// `stub` is the uppercase `x-ss-stub` hex string (absent for GET requests).
/// `cookie` is the exact Cookie header value sent on the wire.
pub fn encrypt(query: &str, stub: Option<&str>, cookie: Option<&str>, ts: i64) -> (String, String) {
    let mut gorgon = [0u8; LENGTH];
    let url_md5 = md5_hex(query.as_bytes());
    for i in 0..4 {
        gorgon[i] = u8::from_str_radix(&url_md5[i * 2..i * 2 + 2], 16).unwrap_or(0);
    }

    if let Some(stub) = stub {
        for i in 0..4 {
            gorgon[4 + i] = u8::from_str_radix(&stub[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
    }

    if let Some(cookie) = cookie {
        let cookie_md5 = md5_hex(cookie.as_bytes());
        for i in 0..4 {
            gorgon[8 + i] = u8::from_str_radix(&cookie_md5[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
    }

    // gorgon[12..16] stays zeroed.
    let khronos = format!("{:x}", ts);
    for i in 0..4 {
        let start = i * 2;
        if start + 2 <= khronos.len() {
            gorgon[16 + i] = u8::from_str_radix(&khronos[start..start + 2], 16).unwrap_or(0);
        }
    }

    (calculate(gorgon), ts.to_string())
}

#[cfg(test)]
mod tests {
    use super::{calculate, encrypt, HEX_STR};

    #[test]
    fn signature_shape() {
        let (gorgon, khronos) = encrypt("aid=1233&device_platform=android", None, None, 1773712689);
        assert!(gorgon.starts_with("8404b4d94000"));
        assert_eq!(gorgon.len(), 52);
        assert_eq!(khronos, "1773712689");
    }

    #[test]
    fn deterministic() {
        assert_eq!(calculate([0u8; 20]), calculate([0u8; 20]));
        assert_eq!(HEX_STR[7], 180);
    }
}
