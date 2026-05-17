{
  pkgs,
  bin,
  lint,
  test-all,
  build,
  coverage,
  shellHook ? "",
}:
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
