# Orkney

## Dependencies

```sh
nix-shell
cargo install wasm-bindgen-cli
```

## Development

```sh
cargo build -p client --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-dir target --target web target/wasm32-unknown-unknown/debug/client.wasm
cargo run -p server
```

## Production

```sh
cargo build -p client --profile wasm-release --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-dir target --target web target/wasm32-unknown-unknown/wasm-release/client.wasm
wasm-opt -Oz --output target/client_bg.wasm target/client_bg.wasm
cargo run -p server
```
