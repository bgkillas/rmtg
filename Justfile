run:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo run --features "debug"
run_tracy:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo run --release --features "tracy,debug"
run_rel:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo run --release
run_full:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo run --profile release_lto
build:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo build --features "debug"
build_rel:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo build --release
build_full:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo build --profile release_lto
miri:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo miri test -- --nocapture --test-threads=1
test:
    export RUSTFLAGS=-Znext-solver=coherence
    cd importer && cargo test -- --nocapture --test-threads=1
test_rel:
    export RUSTFLAGS=-Znext-solver=coherence
    cd importer && cargo test --release -- --nocapture --test-threads=1
bench:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo bench --lib --quiet -- --color always --test-threads=1 --nocapture
clippy:
    export RUSTFLAGS=-Znext-solver=coherence
    cargo fmt
    cargo clippy
wasm:
    export RUSTFLAGS=-Znext-solver=coherence
    cd rmtg && wasm-pack build --no-opt --out-dir ../www/pkg --target web --debug --no-default-features --features "mic,fps"
wasm_rel:
    export RUSTFLAGS=-Znext-solver=coherence
    cd rmtg && wasm-pack build --no-opt --out-dir ../www/pkg --target web --release --no-default-features --features "mic,fps"
    wasm-opt -O4 -all -o www/pkg/rmtg_lib_bg.wasm www/pkg/rmtg_lib_bg.wasm
wasm_full:
    export RUSTFLAGS=-Znext-solver=coherence
    cd rmtg && wasm-pack build --no-opt --out-dir ../www/pkg --target web --profile release_lto --no-default-features --features "mic,fps"
    wasm-opt -O4 -all -o www/pkg/rmtg_lib_bg.wasm www/pkg/rmtg_lib_bg.wasm
run_wasm:
    cd www && python3 -m http.server 8080
update:
    cargo upgrade --incompatible
    cargo update
update_rules:
    cd rules && curl -so rules.txt "$(curl -s 'https://magic.wizards.com/en/rules'|grep media.wizards.com|grep "downloads/MagicCompRules"|grep "\.txt"|sed 's/.*href="//g;s/" .*//;s/ /%20/g')"
    cd rules && dos2unix rules.txt
deploy USER PASS TOTP:
    cd steam && steamcmd +set_steam_guard_code {{TOTP}} +login {{USER}} {{PASS}} +run_app_build "$(pwd)/build.vdf" +quit
