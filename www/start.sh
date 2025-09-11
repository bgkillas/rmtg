set -e
set -x
export RUSTUP_TOOLCHAIN=nightly
wasm-pack build --out-dir www/pkg --target web --release
ls -l pkg/rmtg_bg.wasm
wasm-opt -O4 -all -o pkg/rmtg_bg.wasm pkg/rmtg_bg.wasm
ls -l pkg/rmtg_bg.wasm
if [ $# -ne 0 ]; then
    python3 -m http.server 8080
fi
