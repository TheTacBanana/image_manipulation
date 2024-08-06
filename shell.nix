{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
        pkg-config
        openssl.dev
        glib.dev
        atk.dev
        gtk3.dev

        libGL
        vulkan-headers vulkan-loader
        vulkan-tools vulkan-tools-lunarg
        vulkan-extension-layer
        vulkan-validation-layers
     ];

shellHook = ''
    export RUSTC_VERSION="$(tomlq -r .toolchain.channel rust-toolchain.toml)"
    export PATH="$PATH:''${CARGO_HOME:-~/.cargo}/bin"
    export PATH="$PATH:''${RUSTUP_HOME:-~/.rustup/toolchains/$RUSTC_VERSION-x86_64-unknown-linux/bin}"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
  '';
}