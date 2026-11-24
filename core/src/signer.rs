//! Header assembly and device identity. Mirrors TikTokSigner.generate_headers
//! from iqbalmh18/tiktok-signer with the reference's 37.0.4 constants.

use crate::{argus, gorgon, ladon};
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// All app-version-dependent constants live here. Bump these together when
/// moving to a newer TikTok build; everything else is version agnostic.
pub struct Config;

impl Config {
    pub const AID: u64 = 1233;
    pub const LC_ID: u64 = 2142840551;
    pub const SDK_VER: &'static str = "v05.01.02-alpha.7-ov-android";
    pub const SDK_VER_CODE: u64 = 83952160;
    pub const VERSION_NAME: &'static str = "37.0.4";
    pub const VERSION_CODE: u64 = 2023700040;
}

pub const API_HOST: &str = "api16-normal-c-useast2a.tiktokv.com";
pub const USER_AGENT: &str = "com.zhiliaoapp.musically/2023700040 (Linux; U; Android 9; en_US; 2203121C; Build/PQ3A.190705.09121607;tt-ok/3.12.13.4-tiktok)";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub device_id: String,
    pub iid: String,
    pub openudid: String,
    pub cdid: String,
    pub device_type: String,
    pub device_brand: String,
    pub os_version: String,
    pub channel: String,
    pub resolution: String,
    pub dpi: String,
    pub mcc_mnc: String,
    pub carrier_region: String,
    pub timezone_name: String,
    pub timezone_offset: String,
    pub language: String,
    pub region: String,
    pub locale: String,
}

fn digits(len: usize) -> String {
    let mut rng = rand::rng();
    (0..len)
        .map(|_| rng.random_range(b'0'..=b'9') as char)
        .collect()
}

fn hex_string(len: usize) -> String {
    let mut rng = rand::rng();
    (0..len)
        .map(|_| format!("{:x}", rng.random_range(0..16)))
        .collect()
}

fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

impl DeviceProfile {
    pub fn new() -> Self {
        Self {
            device_id: digits(19),
            iid: digits(19),
            openudid: hex_string(16),
            cdid: uuid_v4(),
            device_type: "2203121C".to_string(),
            device_brand: "Xiaomi".to_string(),
            os_version: "9".to_string(),
            channel: "googleplay".to_string(),
            resolution: "720*1280".to_string(),
            dpi: "320".to_string(),
            mcc_mnc: "310260".to_string(),
            carrier_region: "US".to_string(),
            timezone_name: "America/New_York".to_string(),
            timezone_offset: "-14400".to_string(),
            language: "en".to_string(),
            region: "US".to_string(),
            locale: "en-US".to_string(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid device json: {}", e))
    }
}

impl Default for DeviceProfile {
    fn default() -> Self {
        Self::new()
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn trace_id(device_id: &str, _ts_ms: i64, aid: u64) -> String {
    let device_hex = match device_id.parse::<u128>() {
        Ok(v) => format!("{:x}", v),
        Err(_) => device_id.to_lowercase(),
    };
    let suffix: String = (0..2).map(|_| hex_string(1)).collect();
    let trace = format!("{}{}0{:x}", device_hex, suffix, aid);
    format!("00-{}-{}-01", trace, &trace[..trace.len().min(16)])
}

/// MD5 of a raw POST body, uppercase, matching generate_stub(bytes).
fn stub_for_body(body: &[u8]) -> String {
    use md5::Digest;
    hex::encode(md5::Md5::digest(body)).to_uppercase()
}

/// Generate all authentication headers for a signed TikTok request.
///
/// `query` must be the exact query string used in the URL. `body` is the raw
/// request body for POST requests (absent for GET). `cookie` is the exact
/// Cookie header value, if any.
pub fn generate_headers(
    query: &str,
    body: Option<&[u8]>,
    cookie: Option<&str>,
    device: &DeviceProfile,
    ts: Option<i64>,
) -> Result<BTreeMap<String, String>, String> {
    let ts_ms = ts.unwrap_or_else(now_ms);
    let unix = ts_ms / 1000;

    let mut headers = BTreeMap::new();
    headers.insert("x-ss-req-ticket".to_string(), ts_ms.to_string());
    headers.insert(
        "x-tt-trace-id".to_string(),
        trace_id(&device.device_id, ts_ms, Config::AID),
    );

    let stub = body.map(stub_for_body);
    if let Some(ref stub) = stub {
        headers.insert("x-ss-stub".to_string(), stub.clone());
    }

    let ladon_value = ladon::encrypt(Config::AID, Config::LC_ID, unix);
    headers.insert("x-ladon".to_string(), ladon_value);

    let (gorgon_value, khronos) = gorgon::encrypt(query, stub.as_deref(), cookie, unix);
    headers.insert("x-gorgon".to_string(), gorgon_value);
    headers.insert("x-khronos".to_string(), khronos);

    let argus_value = argus::encrypt(
        query,
        stub.as_deref(),
        unix,
        Config::AID,
        Config::LC_ID,
        Config::SDK_VER,
        Config::SDK_VER_CODE,
        Config::VERSION_NAME,
        &device.device_id,
        &device.device_type,
        &device.os_version,
        &device.channel,
    )?;
    headers.insert("x-argus".to_string(), argus_value);

    if let Some(cookie) = cookie {
        headers.insert("cookie".to_string(), cookie.to_string());
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::{generate_headers, DeviceProfile};

    #[test]
    fn headers_have_expected_shape() {
        let device = DeviceProfile::new();
        let query = "aid=1233&device_platform=android&device_id=1234567890123456789";
        let headers = generate_headers(query, None, None, &device, Some(1773712689000)).unwrap();
        for key in [
            "x-ss-req-ticket",
            "x-tt-trace-id",
            "x-ladon",
            "x-gorgon",
            "x-khronos",
            "x-argus",
        ] {
            assert!(headers.contains_key(key), "missing {}", key);
        }
        assert_eq!(headers["x-khronos"], "1773712689");
        assert_eq!(headers["x-ss-req-ticket"], "1773712689000");
        assert!(headers["x-gorgon"].starts_with("8404b4d94000"));
        assert!(headers["x-argus"].len() > 100);
    }
}
