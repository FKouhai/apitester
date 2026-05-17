{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
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
            overlays = [ (import rust-overlay) ];
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          commonArgs = {
            src = pkgs.lib.cleanSourceWith {
              src = ./.;
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

          hooks = git-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              rustfmt.enable = true;
              clippy.enable = false;
              check-clippy = {
                enable = true;
                name = "clippy";
                entry = "${rustToolchain}/bin/cargo-clippy clippy --no-deps -- -D warnings";
                pass_filenames = false;
                types = [ "rust" ];
              };
              cargo-check.enable = true;
              convco.enable = true;
              nixfmt.enable = true;
              statix.enable = true;
            };
          };
        in
        {
          packages.default = bin;
          devShells.default = import ./shell.nix {
            inherit pkgs bin;
            inherit (hooks) shellHook;
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
