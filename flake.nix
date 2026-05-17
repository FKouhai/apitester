{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    go-overlay = {
      url = "github:purpleclay/go-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      go-overlay,
      crane,
      git-hooks,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      mkForSystem =
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)
              go-overlay.overlays.default
            ];
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./apitester/rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          commonArgs = {
            src = pkgs.lib.cleanSourceWith {
              src = ./apitester;
              filter =
                path: type: (craneLib.filterCargoSources path type) || (pkgs.lib.hasInfix "/tests/fixtures" path);
            };
            nativeBuildInputs = [
              rustToolchain
              pkgs.pkg-config
            ];
            buildInputs = [
              pkgs.openssl
              pkgs.cacert
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          bin = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

          lint = pkgs.writeShellApplication {
            name = "lint";
            runtimeInputs = [ rustToolchain ];
            text = "cargo clippy --no-deps -- -D warnings";
          };
          test-all = pkgs.writeShellApplication {
            name = "test-all";
            runtimeInputs = [ rustToolchain ];
            text = "cargo test";
          };
          build = pkgs.writeShellApplication {
            name = "build";
            runtimeInputs = [ rustToolchain ];
            text = "cargo build";
          };
          coverage = pkgs.writeShellApplication {
            name = "coverage";
            runtimeInputs = [
              rustToolchain
              pkgs.cargo-llvm-cov
            ];
            text = "cargo llvm-cov --open";
          };

          image = pkgs.dockerTools.buildLayeredImage {
            name = "apitester";
            tag = "latest";
            contents = [
              bin
              pkgs.cacert
            ];
            config.Entrypoint = [ "${bin}/bin/apitester" ];
          };

          hooks = git-hooks.lib.${system}.run {
            src = ./apitester;
            hooks = {
              rustfmt = {
                enable = true;
                entry = "${rustToolchain}/bin/rustfmt --edition 2021";
                types = [ "rust" ];
              };
              clippy.enable = false;
              check-clippy = {
                enable = true;
                name = "clippy";
                entry = "${rustToolchain}/bin/cargo-clippy clippy --manifest-path apitester/Cargo.toml --no-deps -- -D warnings";
                pass_filenames = false;
                types = [ "rust" ];
              };
              cargo-check = {
                enable = true;
                entry = "${rustToolchain}/bin/cargo check --manifest-path apitester/Cargo.toml";
                pass_filenames = false;
                types = [ "rust" ];
              };
              convco.enable = true;
              nixfmt.enable = true;
              statix.enable = true;
            };
          };
        in
        {
          packages = {
            default = bin;
            inherit bin image;
          };
          devShells = rec {
            apitester = import ./apitester/shell.nix {
              inherit
                pkgs
                bin
                lint
                test-all
                build
                coverage
                ;
              inherit (hooks) shellHook;
            };
            controller = import ./controller/shell.nix { inherit pkgs; };
            default = apitester;
          };
          checks = {
            inherit hooks;
            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "-- --deny warnings";
              }
            );
            tests = craneLib.cargoTest (
              commonArgs
              // {
                inherit cargoArtifacts;
                SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
              }
            );
          };
        };

      allSystems = builtins.listToAttrs (
        map (system: {
          name = system;
          value = mkForSystem system;
        }) systems
      );
    in
    {
      packages = builtins.mapAttrs (_: s: s.packages) allSystems;
      devShells = builtins.mapAttrs (_: s: s.devShells) allSystems;
      checks = builtins.mapAttrs (_: s: s.checks) allSystems;
    };
}
