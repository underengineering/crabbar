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
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };
  outputs = {
    nixpkgs,
    utils,
    rust-overlay,
    crate2nix,
    ...
  }: let
    name = "crabbar";
  in
    utils.lib.eachDefaultSystem
    (
      system: let
        # Imports
        pkgs = import nixpkgs {inherit system;};
        cargoNixPkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            (self: super: let
              rust = self.rust-bin.stable.latest.default;
            in {
              # Because rust-overlay bundles multiple rust packages into one
              # derivation, specify that mega-bundle here, so that crate2nix
              # will use them automatically.
              rustc = rust;
              cargo = rust;
            })
          ];
        };

        inherit
          (import "${crate2nix}/tools.nix" {inherit pkgs;})
          generatedCargoNix
          ;

        # Create the cargo2nix project
        project =
          cargoNixPkgs.callPackage
          (generatedCargoNix {
            inherit name;
            src = ./.;
          })
          {
            # Individual crate overrides go here
            # Example: https://github.com/balsoft/simple-osd-daemons/blob/6f85144934c0c1382c7a4d3a2bbb80106776e270/flake.nix#L28-L50
            defaultCrateOverrides =
              pkgs.defaultCrateOverrides
              // {
                # The app crate itself is overriden here. Typically we
                # configure non-Rust dependencies (see below) here.
                ${name} = oldAttrs:
                  {
                    inherit buildInputs nativeBuildInputs;
                  }
                  // buildEnvVars;
                gobject-sys = oldAttrs: {
                  buildInputs = [pkgs.gtk4];
                  nativeBuildInputs = [pkgs.pkg-config];
                };
                gio-sys = oldAttrs: {
                  buildInputs = [pkgs.gtk4];
                  nativeBuildInputs = [pkgs.pkg-config];
                };
                gdk-pixbuf-sys = oldAttrs: {
                  buildInputs = [pkgs.gtk4];
                  nativeBuildInputs = [pkgs.pkg-config];
                };
                gtk4-layer-shell-sys = oldAttrs: {
                  buildInputs = [
                    pkgs.gtk4-layer-shell
                  ];
                  nativeBuildInputs = [pkgs.pkg-config];
                };
                libpulse-sys = oldAttrs: {
                  buildInputs = [
                    pkgs.libpulseaudio
                  ];
                  nativeBuildInputs = [pkgs.pkg-config];
                };
                libpulse-mainloop-glib-sys = oldAttrs: {
                  buildInputs = [
                    pkgs.libpulseaudio
                  ];
                  nativeBuildInputs = [pkgs.pkg-config];
                };
              };
          };

        buildInputs = with pkgs; [
          gtk4
          libpulseaudio
          librsvg
        ];
        nativeBuildInputs = with pkgs; [
          cargoNixPkgs.cargo
          cargoNixPkgs.rustc
          gtk4-layer-shell
          pkg-config
        ];
        buildEnvVars = {};
      in rec {
        packages.${name} = project.rootCrate.build;

        # `nix build`
        defaultPackage = packages.${name};

        # `nix run`
        apps.${name} = utils.lib.mkApp {
          inherit name;
          drv = packages.${name};
        };

        defaultApp = apps.${name};

        # `nix develop`
        devShell = let
          rust-dev =
            cargoNixPkgs.rust-bin.stable.latest.default.override
            {
              extensions = ["rust-src" "rust-analyzer"];
            };
        in
          pkgs.mkShell
          {
            inherit buildInputs;
            nativeBuildInputs =
              nativeBuildInputs
              ++ [
                rust-dev
              ];
            RUST_SRC_PATH = "${rust-dev}/lib/rustlib/src/rust/library";
          }
          // buildEnvVars;
      }
    );
}
