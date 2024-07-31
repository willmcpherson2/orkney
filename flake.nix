{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay/b799607";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell rec {
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.gnumake
        ];
        buildInputs = [
          (pkgs.rust-bin.stable."1.79.0".default.override {
            targets = [ "wasm32-unknown-unknown" ];
            extensions = [ "rust-src" "rust-analyzer-preview" ];
          })
          pkgs.concurrently
          pkgs.wasm-bindgen-cli
          pkgs.cargo-watch
          pkgs.binaryen
          pkgs.udev
          pkgs.alsa-lib
          pkgs.vulkan-loader
          pkgs.xorg.libX11
          pkgs.xorg.libXcursor
          pkgs.xorg.libXi
          pkgs.xorg.libXrandr
          pkgs.libxkbcommon
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      };
    };
}
