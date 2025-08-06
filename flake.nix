{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          # rust.stable.latest.default.override {
          #   extensions = [
          #     "rust-src"
          #     "llvm-tools" # required by cargo-llvm-cov
          #   ];
          #   targets = [ "aarch64-unknown-linux-gnu" ];
          # };
          rust.nightly."2025-06-20".default.override {
            extensions = [
              "rust-src"
              "llvm-tools" # required by cargo-llvm-cov
            ];
            targets = [ "aarch64-unknown-linux-gnu" ];
          };
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustToolchain
              cargo-edit # managing cargo dependencies
              cargo-nextest # next-generation test runner
              cargo-llvm-cov # LLVM source-based code coverage
              cargo-lambda # work with AWS Lambda
              bacon # background code checker
              opentofu # alternative to terraform
              git-cliff # generate changelog
              just # just a command runner
              hurl # run and test HTTP requests with plain text
              trivy # find vulnerabilities and misconfigurations
            ];
          };
        }
      );
    };
}
