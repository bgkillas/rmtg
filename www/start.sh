set -e
set -x
export RUSTUP_TOOLCHAIN=nightly
export RUSTFLAGS='--cfg getrandom_backend="wasm_js" -Zunstable-options -Cpanic=immediate-abort'
wasm-pack build --out-dir www/pkg --target web --release --no-default-features --features "wasm"
ls -l pkg/rmtg_bg.wasm
wasm-opt -O4 -all -o pkg/rmtg_bg.wasm pkg/rmtg_bg.wasm
ls -l pkg/rmtg_bg.wasm
if [ $# -ne 0 ]; then
    python3 -m http.server 8080
fi
