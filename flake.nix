{
  description = "Build Kashay";

  inputs = {
    crate2nix.url = "github:nix-community/crate2nix/0.14.0";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-stable.url = "github:NixOS/nixpkgs/23.11";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    flake-utils,
    nixpkgs,
    nixpkgs-stable,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
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
            kashay = attrs: {
              buildInputs = lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.Security
              ];
            };
          };
      };

      kashay = generatedBuild.rootCrate.build;
    in {
      packages = {
        default = kashay;
        kashay = kashay;
      };

      devShells.default = pkgs.mkShell {
        packages =
          [
            pkgs.cargo-nextest
            pkgs.crate2nix
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
