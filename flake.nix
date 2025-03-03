{
  description = "Build eks-creds";

  inputs = {
    crate2nix.url = "github:nix-community/crate2nix/0.14.0";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { flake-utils
    , nixpkgs
    , nixpkgs-stable
    , ...
    } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
    let
      stable-packages = final: _prev: {
        stable = import nixpkgs-stable {
          system = final.system;
          config.allowUnfree = true;
        };
      };
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import inputs.rust-overlay)
          stable-packages
        ];
      };

      generatedBuild = import ./Cargo.nix {
        inherit pkgs;
        defaultCrateOverrides = with pkgs;
          defaultCrateOverrides
          // {
            eks-creds = attrs: {
              buildInputs = lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.Security
              ];
            };
          };
      };

      eks-creds = generatedBuild.rootCrate.build;
    in
    {
      packages = {
        default = eks-creds;
        eks-creds = eks-creds;
      };

      devShells.default = pkgs.mkShell {
        packages =
          [
            pkgs.awscli2
            pkgs.cargo-nextest
            pkgs.crate2nix
            pkgs.cargo-flamegraph
            pkgs.gcc
            pkgs.hyperfine
            pkgs.just
            pkgs.rust-analyzer
            pkgs.rust-bin.stable.latest.default
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];
      };
    });
}
