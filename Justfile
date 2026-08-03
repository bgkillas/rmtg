run:
    cargo run --features "debug"
run_tracy:
    cargo run --release --features "tracy,debug"
run_rel:
    cargo run --release
build:
    cargo build --features "debug"
build_rel:
    cargo build --release
build_full:
    cargo build --profile release_lto
miri:
    cargo miri test -- --nocapture --test-threads=1
test:
    cargo test --quiet -- --nocapture --test-threads=1
test_rel:
    cargo test --release --quiet -- --nocapture --test-threads=1
bench:
    cargo bench --lib --quiet -- --color always --test-threads=1 --nocapture
clippy:
    cargo fmt
    cargo clippy
wasm:
    cd rmtg && wasm-pack build --no-opt --out-dir ../www/pkg --target web --debug
wasm_rel:
    cd rmtg && wasm-pack build --no-opt --out-dir ../www/pkg --target web --release
    wasm-opt -O4 -all -o www/pkg/rmtg_lib_bg.wasm www/pkg/rmtg_lib_bg.wasm
wasm_full:
    cd rmtg && wasm-pack build --no-opt --out-dir ../www/pkg --target web --profile release_lto
    wasm-opt -O4 -all -o www/pkg/rmtg_lib_bg.wasm www/pkg/rmtg_lib_bg.wasm
run_wasm:
    cd www && python3 -m http.server 8080
update:
    cargo upgrade --incompatible
    cargo update
update_rules:
    cd rules && curl -so rules.txt "$(curl -s 'https://magic.wizards.com/en/rules'|grep media.wizards.com|grep "downloads/MagicCompRules"|grep "\.txt"|sed 's/.*href="//g;s/" .*//;s/ /%20/g')"
deploy USER PASS TOTP:
    cd steam && steamcmd +set_steam_guard_code {{TOTP}} +login {{USER}} {{PASS}} +run_app_build "$(pwd)/build.vdf" +quit