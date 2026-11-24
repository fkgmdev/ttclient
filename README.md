# TTClient — anonymous TikTok For You feed (Android)

A minimal Android app that renders TikTok's **For You** feed without an account.
Request signing is done locally in Rust via JNI; there is no hosted signing
service, no API key, and no login.

## Status

- For You feed: `GET /aweme/v1/feed/` works anonymously with locally generated
  signatures (`status_code: 0`, real `aweme_list`).
- Vertical feed UI with autoplay video, author, caption, and stats: implemented.
- Paging: implemented via `max_cursor` with dedup by `aweme_id`.
- Search / profiles / social actions: not implemented (out of scope).

## Layout

| Path | Purpose |
|---|---|
| `core/` | Rust crate with MD5, SM3, Simon, AES-CBC, protobuf, Gorgon, Ladon, Argus, device identity, JNI bridge, and a debug CLI. |
| `android/` | Kotlin + Compose app using OkHttp and Media3. |
| `sdk/` | Local Android SDK / NDK install (not part of source control). |
| `tools/` | Build helpers. |

## Signing references

Ports of the MIT-licensed algorithms from [`iqbalmh18/tiktok-signer`](https://github.com/iqbalmh18/tiktok-signer),
cross-checked against [`tr4cex/TikTok-Encryption`](https://github.com/tr4cex/TikTok-Encryption).

All app-version constants live in `core/src/signer.rs` (`Config`). They currently
match TikTok **37.0.4**. TikTok rotates these: if the API starts rejecting
requests, update `VERSION_NAME`, `VERSION_CODE`, `SDK_VER`, and the matching
query params in `android/.../net/TikTokApi.kt`.

## Build

Prerequisites: JDK 21+, Rust/cargo, Android SDK + NDK (already installed under
`sdk/`), `local.properties` pointing `sdk.dir` at it.

```bash
# 1. Rust unit tests (host)
cd core && cargo test

# 2. Build JNI libs into android/app/src/main/jniLibs/{arm64-v8a,x86_64}
./tools/build-android-core.sh

# 3. Build the APK
cd android && ./gradlew assembleDebug
```

APK: `android/app/build/outputs/apk/debug/app-debug.apk`

## Debug CLI

```bash
cd core
cargo run --bin ttcore-cli -- new-device
cargo run --bin ttcore-cli -- sign --query 'aid=1233&count=6&...' --device '<json>'
cargo run --bin ttcore-cli -- vector
```

## Notes / caveats

- The client is anonymous: device identity is generated locally and persisted in
  `SharedPreferences`. No TikTok account is used or required.
- Private API access depends on IP reputation, app-version constants, and
  device trust. Repeated probing from one IP gets rate-limited (`HTTP 429`) or
  served an HTML challenge with a bogus `Content-Encoding: gzip`, which shows
  up as a JSON parse error. Wait a few minutes before retrying — this is not a
  signing bug (the same failure happens with the Python reference signer).
- The client forces HTTP/1.1 because OkHttp's HTTP/2 streams intermittently
  stall on TikTok's edge servers. Post requests need an `x-ss-stub` body hash;
  the feed is GET-only so no stub is sent.
- This only reads public feed data. Write actions (like/follow/comment) are not
  implemented because they are unreliable without logged-in on-device context.
- Use at your own risk; this is not affiliated with TikTok.
