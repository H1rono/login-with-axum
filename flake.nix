{
  description = "Just login (with axum)";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        inherit (pkgs) lib stdenv;
        toolchain = pkgs.fenix.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-opUgs6ckUQCyDxcB9Wy51pqhd0MPGHUVbwRKKPGiwZU=";
        };
        rustPlatform = pkgs.makeRustPlatform {
          rustc = toolchain;
          cargo = toolchain;
        };
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = with pkgs; [
          openssl
        ] ++ lib.optionals stdenv.isDarwin [
          libiconv
          darwin.Security
        ];
        appBinary = rustPlatform.buildRustPackage {
          pname = "login-with-axum";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          doCheck = false;
          inherit nativeBuildInputs buildInputs;
        };
        publicAssets = stdenv.mkDerivation {
          name = "login-with-axum-public-assets";
          src = ./.;
          phases = [ "unpackPhase" "installPhase" ];
          installPhase = ''
            mkdir -p $out
            cp -R $src/public $out/public
          '';
        };
        appFull = pkgs.symlinkJoin rec {
          pname = "login-with-axum-full";
          version = "0.1.0";
          name = "${pname}-${version}";
          paths = [
            appBinary
            publicAssets
          ];
        };
      in
      {
        packages = {
          default = appFull;
          inherit appBinary appFull publicAssets;
        };

        devShells.default = pkgs.mkShell {
          packages = [ toolchain ] ++ nativeBuildInputs ++ buildInputs;
        };
      });
}
