{
  description = "Rogue-Boi devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" "rustfmt" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            just
            wasm-pack
            cargo-watch
            git-cliff
            bzip2
            nodePackages.npm
            nodePackages.prettier
          ];
        };
      }
    );
}

