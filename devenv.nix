{ pkgs, ... }:

{
  packages = with pkgs; [
    texlive.combined.scheme-medium
  ];
}
