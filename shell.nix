let sources = import ./nix/sources.nix;
in { pkgs ? import sources.nixpkgs {} }: with pkgs;

stdenv.mkDerivation {
  name = "juice-rs";
  buildInputs = [
    rustc
    cargo
    llvmPackages_16.clang
    cmake

    cargo-expand
  ];

  LIBCLANG_PATH="${llvmPackages_16.libclang.lib}/lib";
}
