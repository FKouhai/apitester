{ pkgs }:
let
  go = pkgs.go-bin.fromGoMod ./go.mod;
in
pkgs.mkShell {
  packages = [
    go
    go.withDefaultTools
  ];
}
