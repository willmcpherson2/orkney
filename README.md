# Orkney

## Build

```sh
nix-shell
cargo install wasm-bindgen-cli
cargo build --release --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-name orkney --out-dir target --target web target/wasm32-unknown-unknown/release/orkney.wasm
python3 -m http.server
```
