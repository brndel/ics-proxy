{
  description = "An ics proxy that can modify events using json config";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

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
      ...
    }:
    {
      nixosModules.default =
        { config, lib, ... }:
        let
          pkgs = import <nixpkgs> {
            overlays = [
              rust-overlay.overlays.default
            ];
          };
        in
        import ./. {
          inherit pkgs lib config;
        };
    };
}
