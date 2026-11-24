//! X-Ladon generation, ported from iqbalmh18/tiktok-signer.

use rand::Rng;

#[inline]
fn ror(v: u64, count: u32) -> u64 {
    let count = count % 64;
    if count == 0 {
        v
    } else {
        v.rotate_right(count)
    }
}

fn key_schedule(md5hex: &[u8]) -> [u8; 288] {
    let mut table = [0u8; 288];
    table[..32].copy_from_slice(md5hex);

    let mut temp: Vec<u64> = Vec::with_capacity(4);
    for chunk in table[..32].chunks_exact(8) {
        temp.push(u64::from_le_bytes(chunk.try_into().unwrap()));
    }
    let mut buffer_b0 = temp[0];
    let mut buffer_b8 = temp[1];
    temp.clear();

    for i in 0..0x22 {
        let x9 = buffer_b0;
        let mut x8 = ror(buffer_b8, 8);
        x8 = x8.wrapping_add(x9);
        x8 ^= i as u64;
        temp.push(x8);
        x8 ^= ror(x9, 61);
        let start = i * 8 + 8;
        table[start..start + 8].copy_from_slice(&x8.to_le_bytes());
        buffer_b0 = x8;
        buffer_b8 = temp.remove(0);
    }
    table
}

fn encrypt_block(table: &[u8; 288], block: &[u8; 16]) -> [u8; 16] {
    let mut data0 = u64::from_le_bytes(block[..8].try_into().unwrap());
    let mut data1 = u64::from_le_bytes(block[8..].try_into().unwrap());
    for i in 0..0x22 {
        let hash_val = u64::from_le_bytes(table[i * 8..i * 8 + 8].try_into().unwrap());
        data1 = hash_val ^ data0.wrapping_add(ror(data1, 8));
        data0 = data1 ^ data0.rotate_left(3);
    }
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&data0.to_le_bytes());
    out[8..].copy_from_slice(&data1.to_le_bytes());
    out
}

/// Generate the `x-ladon` header value.
pub fn encrypt(aid: u64, lc_id: u64, ts: i64) -> String {
    let plaintext = format!("{}-{}-{}", ts, lc_id, aid);
    let data = plaintext.as_bytes();
    let padded_len = ((data.len() + 15) / 16) * 16;
    let mut input = vec![0u8; padded_len];
    input[..data.len()].copy_from_slice(data);
    let pad_value = (padded_len - data.len()) as u8;
    for byte in input.iter_mut().skip(data.len()) {
        *byte = pad_value;
    }

    let mut rand_bytes = [0u8; 4];
    rand::rng().fill_bytes(&mut rand_bytes);
    let mut keygen = Vec::with_capacity(4 + 20);
    keygen.extend_from_slice(&rand_bytes);
    keygen.extend_from_slice(aid.to_string().as_bytes());
    use md5::Digest;
    let md5hex = hex::encode(md5::Md5::digest(&keygen));
    let table = key_schedule(md5hex.as_bytes());

    let mut output = Vec::with_capacity(padded_len + 4);
    output.extend_from_slice(&rand_bytes);
    for block in input.chunks_exact(16) {
        let block: [u8; 16] = block.try_into().unwrap();
        output.extend_from_slice(&encrypt_block(&table, &block));
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(&output)
}

#[cfg(test)]
mod tests {
    use super::encrypt;
    use base64::Engine;

    #[test]
    fn decodes_to_expected_length() {
        // plaintext "1773712689-2142840551-1233" is 26 bytes -> padded to 32, +4 random = 36
        let value = encrypt(1233, 2142840551, 1773712689);
        let raw = base64::engine::general_purpose::STANDARD
            .decode(value)
            .unwrap();
        assert_eq!(raw.len(), 36);
    }
}
