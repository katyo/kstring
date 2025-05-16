{ pkgs ? import <nixpkgs> {
    overlays = [ (import <rust-overlay>) (import <nixos-addons/packages>) ];
}, ... }:
with pkgs;
let
    llvmPackages = llvmPackages_latest;
    stdenv = llvmPackages.stdenv;
    rust = rust-bin.stable.latest.default.override (attrs: {
        extensions = attrs.extensions ++ [
            "rust-src"
            "rust-analyzer"
            #"miri"
        ];
        targets = attrs.targets ++ [
            #"x86_64-unknown-linux-gnu"
            #"x86_64-unknown-linux-musl"
            #"x86_64-pc-windows-gnu"
        ];
    });
    rustNightly = rust-bin.nightly.latest.default.override (attrs: {
        extensions = attrs.extensions ++ [
            "rust-analysis"
            "rust-src"
            "miri"
        ];
        targets = attrs.targets ++ [
            #"x86_64-unknown-linux-gnu"
            #"x86_64-unknown-linux-musl"
            #"x86_64-pc-windows-gnu"
        ];
    });
in stdenv.mkDerivation {
    name = "dev-shell";
    nativeBuildInputs = with llvmPackages; [
        pkg-config
        #rust
        rustNightly
        clang
    ];
}
