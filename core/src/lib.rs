pub mod argus;
pub mod gorgon;
pub mod ladon;
pub mod protobuf;
pub mod signer;
pub mod simon;
pub mod sm3;

use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString, Reference};
use jni::{Env, EnvUnowned};

#[allow(deprecated)]
fn read_string(env: &Env, value: &JString) -> Option<String> {
    if value.is_null() {
        return None;
    }
    env.get_string(value).ok().map(|s| s.into())
}

/// JNI: create a fresh device identity as JSON.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_local_ttclient_core_Signer_newDevice<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<JString> {
        JString::from_str(env, signer::DeviceProfile::new().to_json())
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}

/// JNI: sign a request. Returns a JSON object of header name -> value, or a
/// JSON object with an "error" key when signing fails.
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_local_ttclient_core_Signer_sign<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    query: JString<'caller>,
    body: JString<'caller>,
    cookie: JString<'caller>,
    device: JString<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<JString> {
        let query = read_string(env, &query).unwrap_or_default();
        let body = read_string(env, &body);
        let cookie = read_string(env, &cookie);
        let device_json = read_string(env, &device).unwrap_or_default();

        let json = match signer::DeviceProfile::from_json(&device_json) {
            Ok(profile) => {
                let result = signer::generate_headers(
                    &query,
                    body.as_deref().map(|s| s.as_bytes()),
                    cookie.as_deref(),
                    &profile,
                    None,
                );
                match result {
                    Ok(headers) => serde_json::to_string(&headers)
                        .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() }).to_string()),
                    Err(e) => serde_json::json!({ "error": e }).to_string(),
                }
            }
            Err(e) => serde_json::json!({ "error": e }).to_string(),
        };
        JString::from_str(env, json)
    });
    outcome.resolve::<ThrowRuntimeExAndDefault>()
}
