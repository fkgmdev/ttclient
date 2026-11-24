//! Minimal protobuf writer matching iqbalmh18/tiktok-signer's ProtoBuf encoder:
//! ints -> varint, strings/bytes -> length-delimited, nested maps -> submessage.

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Value {
    Varint(u64),
    Bytes(Vec<u8>),
    Nested(BTreeMap<u32, Value>),
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Varint(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Varint(v as u64)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Varint(v as u64)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Bytes(v.as_bytes().to_vec())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Bytes(v.into_bytes())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(v)
    }
}

fn write_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push(((v & 0x7f) | 0x80) as u8);
        v >>= 7;
    }
    out.push((v & 0x7f) as u8);
}

/// Encode a field map. Uses a BTreeMap so fields are emitted in ascending field
/// number, which matches the insertion order used by the Python reference.
pub fn encode(fields: &BTreeMap<u32, Value>) -> Vec<u8> {
    let mut out = Vec::new();
    for (&idx, value) in fields {
        match value {
            Value::Varint(v) => {
                write_varint(&mut out, ((idx << 3) | 0) as u64);
                write_varint(&mut out, *v);
            }
            Value::Bytes(b) => {
                write_varint(&mut out, ((idx << 3) | 2) as u64);
                write_varint(&mut out, b.len() as u64);
                out.extend_from_slice(b);
            }
            Value::Nested(nested) => {
                write_varint(&mut out, ((idx << 3) | 2) as u64);
                let payload = encode(nested);
                write_varint(&mut out, payload.len() as u64);
                out.extend_from_slice(&payload);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{encode, Value};
    use std::collections::BTreeMap;

    #[test]
    fn varint_boundary() {
        let mut f = BTreeMap::new();
        f.insert(1, Value::Varint(128));
        assert_eq!(encode(&f), vec![0x08, 0x80, 0x01]);
    }

    #[test]
    fn string_field() {
        let mut f = BTreeMap::new();
        f.insert(1, Value::from("ok"));
        assert_eq!(encode(&f), vec![0x0a, 0x02, b'o', b'k']);
    }
}
