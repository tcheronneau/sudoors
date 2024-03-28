{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
  let
    name = "sudoors";
    version = "0.1.0";
  in
  flake-utils.lib.eachDefaultSystem (system:
    with nixpkgs.legacyPackages.${system}; {
      packages.sudoors = rustPlatform.buildRustPackage {
        name = "${name}";
        version = "${version}";

        src = lib.cleanSource ./.;

        cargoSha256 =
          "sha256-PBZesxydlEwZkrHNLuPLM4/rchIRRojQCu5cx6VE6VM=";
        nativeBuildInputs = [
          rustc
          cargo
          pkg-config
          openssl.dev
          sqlite
          protobuf
        ];
        buildInputs = [
          openssl.dev
          sqlite
          protobuf
        ];
      };
      packages.docker = dockerTools.buildLayeredImage {
        name = "mcth/${name}";
        contents = [ self.packages.${system}.sudoors  cacert ];
        tag = "${system}-${version}";
        created = "now";
        config = {
          Cmd = [
            "${self.packages.${system}.sudoors}/bin/${name}"
          ];
        };
      };
      defaultPackage = self.packages.${system}.sudoors;
      devShell = mkShell {
        inputsFrom = builtins.attrValues self.packages.${system};

        buildInputs = [
          rustc
          rustfmt
          rust-analyzer
          cargo
          pkg-config
          openssl.dev
          protobuf
          diesel-cli
          grpcurl
          sqlite
        ];
      };
    });
}
