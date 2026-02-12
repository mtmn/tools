{
  description = "miro@amen";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-index-database = {
      url = "github:nix-community/nix-index-database";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      nixpkgs,
      home-manager,
      nix-index-database,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      homeConfigurations."miro" = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;
        modules = [
          nix-index-database.homeModules.default
          ./tools.nix
          {
            home.username = "miro";
            home.homeDirectory = "/home/miro";
            home.stateVersion = "25.11";
            programs.home-manager.enable = true;
          }
        ];
      };
    };
}
