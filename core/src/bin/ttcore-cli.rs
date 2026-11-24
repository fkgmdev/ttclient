use std::collections::BTreeMap;
use ttcore::signer;

fn usage() {
    eprintln!(
        "usage:\n  ttcore-cli new-device\n  ttcore-cli sign --query <query> [--body <body>] [--cookie <cookie>] [--device <json>] [--ts <ms>]\n  ttcore-cli vector"
    );
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "new-device" => println!("{}", signer::DeviceProfile::new().to_json()),
        "sign" => {
            let query = arg_value(&args, "--query").unwrap_or_default();
            let body = arg_value(&args, "--body");
            let cookie = arg_value(&args, "--cookie");
            let device_json = arg_value(&args, "--device").unwrap_or_default();
            let ts = arg_value(&args, "--ts")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or_else(signer::now_ms);
            let device = signer::DeviceProfile::from_json(&device_json)
                .unwrap_or_else(|_| signer::DeviceProfile::new());
            match signer::generate_headers(
                &query,
                body.as_deref().map(|s| s.as_bytes()),
                cookie.as_deref(),
                &device,
                Some(ts),
            ) {
                Ok(headers) => println!("{}", serde_json::to_string(&headers).unwrap()),
                Err(e) => {
                    eprintln!("sign failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "vector" => {
            let device = signer::DeviceProfile::from_json(
                r#"{"device_id":"1234567890123456789","iid":"7671702915996206855","openudid":"1234567890abcdef","cdid":"7f3b4b5b-1111-4222-8333-444455556666","device_type":"2203121C","device_brand":"Xiaomi","os_version":"9","channel":"googleplay","resolution":"720*1280","dpi":"320","mcc_mnc":"310260","carrier_region":"US","timezone_name":"America/New_York","timezone_offset":"-14400","language":"en","region":"US","locale":"en-US"}"#,
            )
            .unwrap();
            let query = "aid=1233&app_name=musical_ly&channel=googleplay&count=6&device_id=1234567890123456789&device_platform=android&device_type=2203121C&max_cursor=0&os_version=9&type=0&version_name=37.0.4";
            let headers: BTreeMap<String, String> =
                signer::generate_headers(query, None, None, &device, Some(1773712689000)).unwrap();
            println!("{}", serde_json::to_string_pretty(&headers).unwrap());
        }
        _ => usage(),
    }
}
