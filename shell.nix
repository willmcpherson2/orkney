let
  rust-overlay = import
    (fetchTarball
      "https://github.com/oxalica/rust-overlay/archive/e36f66bb10b09f5189dc3b1706948eaeb9a1c555.tar.gz");
  wasm-bindgen-cli = import
    (fetchTarball
      "https://github.com/NixOS/nixpkgs/archive/9957cd48326fe8dbd52fdc50dd2502307f188b0d.tar.gz")
    { };
in
{ pkgs ? import <nixpkgs> { overlays = [ rust-overlay ]; } }:
pkgs.mkShell rec {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    (pkgs.writeShellScriptBin "watch-server" ''
      cargo watch -x "run -p server"
    '')
    (pkgs.writeShellScriptBin "watch-client" ''
      cargo watch \
        -x "build -p client --profile wasm-dev --target wasm32-unknown-unknown" \
        -s "wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/wasm-dev/client.wasm"
    '')
    (pkgs.writeShellScriptBin "watch" ''
      concurrently -n server,client -c red,blue watch-server watch-client
    '')
    (pkgs.writeShellScriptBin "build" ''
      cargo build -p server --profile release
      cargo build -p client --profile wasm-release --target wasm32-unknown-unknown
      wasm-bindgen --out-dir target --target web --no-typescript target/wasm32-unknown-unknown/wasm-release/client.wasm
      wasm-opt -Oz --output target/client_bg.wasm target/client_bg.wasm
    '')
    (pkgs.rust-bin.stable."1.75.0".default.override {
      targets = [ "wasm32-unknown-unknown" ];
      extensions = [ "rust-src" "rust-analyzer-preview" ];
    })
    pkgs.concurrently
    wasm-bindgen-cli.wasm-bindgen-cli
    pkgs.cargo-watch
    pkgs.binaryen
    pkgs.udev
    pkgs.alsa-lib
    pkgs.vulkan-loader
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXi
    pkgs.xorg.libXrandr
  ];
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
