{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixpkgs-unstable";
    };
    utils = {
      url = "github:numtide/flake-utils";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
    utils,
    rust-overlay,
    ...
  }:
    utils.lib.eachDefaultSystem
    (
      system: let
        # Imports
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
      in {
        packages = let
          rust = pkgs.rust-bin.stable.latest.default;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
        in {
          crabbar = rustPlatform.buildRustPackage {
            pname = "crabbar";
            version = version;

            cargoLock.lockFile = ./Cargo.lock;
            src = ./.;

            nativeBuildInputs = with pkgs; [pkg-config];
            buildInputs = with pkgs; [gtk4 gtk4-layer-shell libpulseaudio librsvg];
          };
        };

        devShell = let
          rust-dev =
            pkgs.rust-bin.stable.latest.default.override
            {
              extensions = ["rust-src" "rust-analyzer"];
            };
        in
          pkgs.mkShell
          {
            inputsFrom = [self.packages.${system}.crabbar];
            packages = [rust-dev];
            RUST_SRC_PATH = "${rust-dev}/lib/rustlib/src/rust/library";
          };
      }
    );
}
