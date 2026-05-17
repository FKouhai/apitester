{
  pkgs,
  bin,
  shellHook ? "",
}:
pkgs.mkShell {
  inputsFrom = [ bin ];
  packages = with pkgs; [
    cargo-llvm-cov
    cargo-watch
    clippy
  ];
  inherit shellHook;
}
