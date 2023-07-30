let
  rust_overlay = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
  rust_channel = nixpkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
with nixpkgs;

# Define the shell
pkgs.mkShell {
  nativeBuildInputs = [
    nixpkgs.bzip2
    nodePackages.yarn
    rust_channel # Full rust from overlay, includes cargo
    wasm-pack
  ];
  installPhase = ''
    ${nixpkgs.bzip2.postInstall}
  '';
  shellHook = ''
    yarn
  '';
}
