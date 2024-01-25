# Orkney

## Dependencies

```sh
nix-shell
cargo install wasm-bindgen-cli
```

## Development

```sh
cargo build -p client --profile wasm-dev --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/wasm-dev/client.wasm
cargo run -p server
```

## Watch

```sh
~/.cargo/bin/cargo-watch -x 'run -p server'
~/.cargo/bin/cargo-watch -x 'build -p client --profile wasm-dev --target wasm32-unknown-unknown' -s '~/.cargo/bin/wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/wasm-dev/client.wasm'
```

## Production

```sh
cargo build -p client --profile wasm-release --target wasm32-unknown-unknown
~/.cargo/bin/wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/wasm-release/client.wasm
wasm-opt -Oz --output target/client_bg.wasm target/client_bg.wasm
cargo build -p server --profile release
```
