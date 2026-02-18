{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # System libraries required by Tauri v2 on Linux
        libraries = with pkgs; [
          alsa-lib
          cairo
          gdk-pixbuf
          glib
          gtk3
          libsoup_3
          openssl
          pango
          webkitgtk_4_1
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cmake
            llvmPackages.libclang
            pkg-config
            gobject-introspection
          ];

          buildInputs = libraries;

          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            nodejs
            pnpm
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        };
      });
}
