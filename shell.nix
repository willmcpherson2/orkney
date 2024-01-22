{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [
    pkgs.pkg-config
    pkgs.alsa-lib
    pkgs.libudev-zero
  ];
}
