{
  description = "A C compiler written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        riscvBinutils = pkgs.pkgsCross.riscv64.buildPackages.binutils;
        riscvLinuxGnuBinutils = pkgs.runCommand "riscv64-linux-gnu-binutils" { } ''
          mkdir -p $out/bin
          ln -s ${riscvBinutils}/bin/riscv64-unknown-linux-gnu-as $out/bin/riscv64-linux-gnu-as
          ln -s ${riscvBinutils}/bin/riscv64-unknown-linux-gnu-ld $out/bin/riscv64-linux-gnu-ld
        '';
        treefmtEval = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            nixfmt.enable = true;
            rustfmt.enable = true;
            taplo.enable = true;
          };
        };
      in
      {
        formatter = treefmtEval.config.build.wrapper;
        checks.formatting = treefmtEval.config.build.check self;

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          doCheck = true;

          nativeCheckInputs = [
            pkgs.qemu
            riscvLinuxGnuBinutils
          ];

          preCheck = ''
            patchShebangs scripts/run-rv32.sh
          '';
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            cargo-nextest

            qemu
            riscvLinuxGnuBinutils
          ];
        };
      }
    );
}
