#!/usr/bin/env bash
# Cross-compile the Rust signing core into JNI libraries for Android and copy
# them into the app's jniLibs directory.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NDK="${ANDROID_NDK_HOME:-$ROOT/sdk/ndk/27.3.13750724}"
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin"
JNI_LIBS="$ROOT/android/app/src/main/jniLibs"

if [[ ! -d "$TOOLCHAIN" ]]; then
  echo "error: NDK toolchain not found at $TOOLCHAIN" >&2
  exit 1
fi

export AR="$TOOLCHAIN/llvm-ar"
export RANLIB="$TOOLCHAIN/llvm-ranlib"

build_target() {
  local rust_target="$1" abi="$2"
  echo "==> building $rust_target ($abi)"
  (
    cd "$ROOT/core"
    cargo build --target "$rust_target" --release --lib
  )
  mkdir -p "$JNI_LIBS/$abi"
  cp "$ROOT/core/target/$rust_target/release/libttcore.so" "$JNI_LIBS/$abi/libttcore.so"
}

build_target aarch64-linux-android arm64-v8a
build_target x86_64-linux-android  x86_64

echo "==> JNI libraries written to $JNI_LIBS"
ls -lh "$JNI_LIBS"/*/libttcore.so
