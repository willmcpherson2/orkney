.PHONY: watch-server watch-client-native watch-client-web watch-native watch-web build-server build-client-native build-client-web build-native build-web

watch-server:
	cargo watch -x "run -p server --color always"

watch-client-native:
	cargo watch -x "run -p client --profile native-dev --features bevy/dynamic_linking --color always"

watch-client-web:
	cargo watch \
		-x "build -p client --profile web-dev --target wasm32-unknown-unknown --color always" \
		-s "wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/web-dev/client.wasm"

watch-native:
	concurrently -n server,client -c red,blue "make watch-server" "make watch-client-native"

watch-web:
	concurrently -n server,client -c red,blue "make watch-server" "make watch-client-web"

build-server:
	cargo build -p server --profile release

build-client-native:
	cargo build -p client --profile release

build-client-web:
	cargo build -p client --profile web-release --target wasm32-unknown-unknown && \
	wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/web-release/client.wasm && \
	wasm-opt -Oz --output target/client_bg.wasm target/client_bg.wasm

build-native: build-server build-client-native

build-web: build-server build-client-web
