{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
	flake-utils.follows = "flake-utils";
	nixpkgs.follows = "nixpkgs";
      };
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, crane, fenix, flake-utils, nixpkgs }:
    let nightlyToolchain = (import ./toolchain.nix { crane=crane; fenix=fenix; }).mkToolchain; in
    flake-utils.lib.eachDefaultSystem (system: 
    let pkgs = import nixpkgs { inherit system; }; in
    {
      packages.default = let craneLib = (nightlyToolchain system);
      in
      craneLib.buildPackage {
	src = ./.;
	buildInputs = with pkgs; [ libiconv ];
      };
    });
}
