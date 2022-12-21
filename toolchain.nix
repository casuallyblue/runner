{ crane, fenix, ... }: {
  mkToolchain = system: 
    (crane.lib.${system}.overrideToolchain 
      	fenix.packages.${system}.complete.toolchain
    );
}
