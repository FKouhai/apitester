{
  pkgs,
  bin,
  shellHook ? "",
}:
let
  rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
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
in
pkgs.mkShell {
  inputsFrom = [ bin ];
  packages = with pkgs; [
    cargo-llvm-cov
    cargo-watch
    clippy
    lint
    test-all
    build
    coverage
  ];
  inherit shellHook;
}
