//! X-Argus generation, ported from iqbalmh18/tiktok-signer.

use crate::protobuf::{encode as pb_encode, Value};
use crate::simon;
use crate::sm3;
use aes::Aes128;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use block_padding::Pkcs7;
use cbc::Encryptor;
use cipher::{BlockModeEncrypt, KeyIvInit};
use rand::RngExt;
use std::collections::BTreeMap;

const SIGN_KEY: &[u8; 32] =
    b"\xac\x1a\xda\xae\x95\xa7\xaf\x94\xa5\x11J\xb3\xb3\xa9}\xd8\x00P\xaa\n91L@R\x8c\xae\xc9RV\xc2\x8c";
const SM3_OUTPUT: &[u8; 32] =
    b"\xfcx\xe0\xa9ez\x0ct\x8c\xe5\x15Y\x90<\xcf\x03Q\x0eQ\xd3\xcf\xf22\xd7\x13C\xe8\x8a2\x1cS\x04";
const PREFIX: &[u8; 8] = b"\xf2\xf7\xfc\xff\xf2\xf7\xfc\xff";
const WRAP_HEAD: &[u8; 9] = b"\xa6n\xad\x9fw\x01\xd0\x0c\x18";
const WRAP_TAIL: &[u8; 2] = b"ao";

fn md5(data: &[u8]) -> [u8; 16] {
    use md5::Digest;
    md5::Md5::digest(data).into()
}

fn body_hash(stub: Option<&str>) -> [u8; 6] {
    let data: Vec<u8> = match stub {
        Some(s) if !s.is_empty() => hex::decode(s).unwrap_or_else(|_| vec![0u8; 16]),
        _ => vec![0u8; 16],
    };
    let digest = sm3::digest(&data);
    let mut out = [0u8; 6];
    out.copy_from_slice(&digest[..6]);
    out
}

fn query_hash(query: &str) -> [u8; 6] {
    let data = if query.is_empty() {
        vec![0u8; 16]
    } else {
        query.as_bytes().to_vec()
    };
    let digest = sm3::digest(&data);
    let mut out = [0u8; 6];
    out.copy_from_slice(&digest[..6]);
    out
}

fn app_version_hash(version_name: &str) -> u64 {
    let mut parts: Vec<u32> = version_name
        .split('.')
        .map(|p| p.parse::<u32>().unwrap_or(0))
        .collect();
    while parts.len() < 3 {
        parts.push(0);
    }
    let hex = format!("{:x}{:x}{:x}00", parts[2] * 4, parts[1] * 16, parts[0] * 4);
    let padded = format!("{:0>8}", hex);
    let bytes = hex::decode(padded).unwrap_or_else(|_| vec![0u8; 4]);
    let mut buf = [0u8; 4];
    let n = bytes.len().min(4);
    buf[..n].copy_from_slice(&bytes[..n]);
    (u32::from_be_bytes(buf) as u64) << 1
}

/// Minimal form-style lookup matching Python's `parse_qs(...)[key][0]`.
fn query_value(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        if k == key {
            return Some(v.replace('+', " "));
        }
    }
    None
}

fn pkcs7_padding(data: &[u8], block: usize) -> Vec<u8> {
    let pad = block - (data.len() % block);
    let mut out = data.to_vec();
    out.extend(std::iter::repeat(pad as u8).take(pad));
    out
}

fn calculate_xargus(bean: &BTreeMap<u32, Value>) -> Result<String, String> {
    let pb = pb_encode(bean);
    let padded = pkcs7_padding(&pb, 16);

    let mut key = [0u64; 4];
    for i in 0..4 {
        key[i] = u64::from_le_bytes(SM3_OUTPUT[i * 8..i * 8 + 8].try_into().unwrap());
    }

    let mut enc_pb = vec![0u8; padded.len()];
    for (i, block) in padded.chunks_exact(16).enumerate() {
        let pt = (
            u64::from_le_bytes(block[..8].try_into().unwrap()),
            u64::from_le_bytes(block[8..].try_into().unwrap()),
        );
        let (c0, c1) = simon::encrypt(pt, &key);
        enc_pb[i * 16..i * 16 + 8].copy_from_slice(&c0.to_le_bytes());
        enc_pb[i * 16 + 8..i * 16 + 16].copy_from_slice(&c1.to_le_bytes());
    }

    let mut data = Vec::with_capacity(8 + enc_pb.len());
    data.extend_from_slice(PREFIX);
    data.extend_from_slice(&enc_pb);
    let mut scrambled = data.clone();
    for i in 8..data.len() {
        scrambled[i] ^= data[i % 8];
    }
    scrambled.reverse();

    let mut wrapped = Vec::with_capacity(WRAP_HEAD.len() + scrambled.len() + WRAP_TAIL.len());
    wrapped.extend_from_slice(WRAP_HEAD);
    wrapped.extend_from_slice(&scrambled);
    wrapped.extend_from_slice(WRAP_TAIL);
    let padded_wrapped = pkcs7_padding(&wrapped, 16);

    let aes_key = md5(&SIGN_KEY[..16]);
    let aes_iv = md5(&SIGN_KEY[16..]);
    let cipher = Encryptor::<Aes128>::new_from_slices(&aes_key, &aes_iv)
        .map_err(|e| format!("aes init: {}", e))?;
    let ciphertext = cipher.encrypt_padded_vec::<Pkcs7>(&padded_wrapped);

    let mut out = Vec::with_capacity(2 + ciphertext.len());
    out.extend_from_slice(&[0xf2, 0x81]);
    out.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&out))
}

/// Generate the `x-argus` header value.
#[allow(clippy::too_many_arguments)]
pub fn encrypt(
    query: &str,
    stub: Option<&str>,
    ts: i64,
    aid: u64,
    lc_id: u64,
    sdk_ver: &str,
    sdk_ver_code: u64,
    version_name: &str,
    device_id: &str,
    device_type: &str,
    os_version: &str,
    channel: &str,
) -> Result<String, String> {
    let channel = query_value(query, "channel").unwrap_or_else(|| channel.to_string());
    let device_id = query_value(query, "device_id").unwrap_or_else(|| device_id.to_string());
    let device_type = query_value(query, "device_type").unwrap_or_else(|| device_type.to_string());
    let os_version = query_value(query, "os_version").unwrap_or_else(|| os_version.to_string());
    let version_name =
        query_value(query, "version_name").unwrap_or_else(|| version_name.to_string());

    let mut bean = BTreeMap::new();
    bean.insert(1, Value::Varint(0x2020_0929u64 << 1));
    bean.insert(2, Value::Varint(2));
    bean.insert(3, Value::Varint(rand::rng().random_range(0..=0x7fff_ffff)));
    bean.insert(4, Value::from(aid.to_string()));
    bean.insert(5, Value::from(device_id.clone()));
    bean.insert(6, Value::from(lc_id.to_string()));
    bean.insert(7, Value::from(sdk_ver));
    bean.insert(8, Value::from(sdk_ver));
    bean.insert(9, Value::Varint(sdk_ver_code));
    bean.insert(10, Value::Bytes(vec![0u8; 8]));
    bean.insert(11, Value::from("android"));
    bean.insert(12, Value::Varint((ts as u64) << 1));
    bean.insert(13, Value::Bytes(body_hash(stub).to_vec()));
    bean.insert(14, Value::Bytes(query_hash(query).to_vec()));

    let mut nested15 = BTreeMap::new();
    nested15.insert(1, Value::Varint(85));
    nested15.insert(2, Value::Varint(85));
    nested15.insert(3, Value::Varint(85));
    nested15.insert(5, Value::Varint(85));
    nested15.insert(6, Value::Varint(170));
    nested15.insert(7, Value::Varint(((ts as u64) << 1).wrapping_sub(310)));
    bean.insert(15, Value::Nested(nested15));

    bean.insert(16, Value::from(device_id));
    bean.insert(20, Value::from("none"));
    bean.insert(21, Value::Varint(738));

    let mut nested23 = BTreeMap::new();
    nested23.insert(1, Value::from(device_type));
    nested23.insert(2, Value::from(os_version));
    nested23.insert(3, Value::from(channel));
    nested23.insert(4, Value::Varint(app_version_hash(&version_name)));
    bean.insert(23, Value::Nested(nested23));

    bean.insert(25, Value::Varint(2));

    calculate_xargus(&bean)
}

#[cfg(test)]
mod tests {
    use super::app_version_hash;

    #[test]
    fn version_hash_matches_reference() {
        // Python reference: 0x01009400 big-endian, shifted left by one.
        assert_eq!(app_version_hash("37.0.4"), 33_630_208);
    }
}
