# Orkney

## Build

```sh
nix-shell
cargo install wasm-bindgen-cli
cargo build --profile wasm-release --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-name orkney --out-dir target --target web target/wasm32-unknown-unknown/wasm-release/orkney.wasm
wasm-opt -Oz --output target/orkney_bg.wasm target/orkney_bg.wasm
python3 -m http.server
```
