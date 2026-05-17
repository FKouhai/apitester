{ pkgs }:
let
  go = pkgs.go-bin.fromGoMod ./go.mod;
  lint = pkgs.writeShellApplication {
    name = "lint";
    runtimeInputs = [ go ];
    text = "go vet ./...";
  };
  test-all = pkgs.writeShellApplication {
    name = "test-all";
    runtimeInputs = [ go ];
    text = "go test ./...";
  };
  build = pkgs.writeShellApplication {
    name = "build";
    runtimeInputs = [ go ];
    text = "go build ./...";
  };
in
pkgs.mkShell {
  packages = [
    go
    go.withDefaultTools
    lint
    test-all
    build
  ];
}
