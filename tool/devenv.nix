{ pkgs, lib, ... }:

let
  # FlowCutter PACE 2017 — heuristic treewidth solver (runs until SIGTERM,
  # then outputs best decomposition found). Not packaged in nixpkgs.
  # Source: https://github.com/kit-algo/flow-cutter-pace17
  #
  # To obtain the correct sha256, run:
  #   nix-prefetch-github kit-algo flow-cutter-pace17 --rev master
  # then replace lib.fakeHash below with the printed value.
  # TODO cite using the code in paper.
  flow-cutter-pace17 = pkgs.stdenv.mkDerivation {
    pname = "flow-cutter-pace17";
    version = "unstable-2017";

    src = pkgs.fetchFromGitHub {
      owner = "kit-algo";
      repo = "flow-cutter-pace17";
      rev = "master";
      sha256 = "sha256-FYzHiFTi6oJpkWrFhjRHKdtNkisIw5Mc4Fq4ZtXH94g=";
    };

    # Only GCC needed — FlowCutter has no other dependencies (see README).
    # bash is provided by stdenv and does not need to be listed explicitly.
    nativeBuildInputs = [ pkgs.gcc ];

    buildPhase = ''
      bash build.sh
    '';

    installPhase = ''
      mkdir -p $out/bin
      cp flow_cutter_pace17 $out/bin/
    '';

    meta = {
      description = "FlowCutter heuristic treewidth solver (PACE 2017)";
      homepage = "https://github.com/kit-algo/flow-cutter-pace17";
      license = lib.licenses.bsd2;
    };
  };
in
{
  # Read toolchain, channel, and components (including rustc-dev) from
  # rust-toolchain.toml. This is the canonical way to pin nightly + extras.
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  packages = [ flow-cutter-pace17 ];

  env = {
    # Make the solver binary name explicit for the treewidth crate.
    FLOW_CUTTER_BIN = "flow_cutter_pace17";
  };

  # rustc-dev ships rustc's internal libraries as shared objects. They must be
  # on LD_LIBRARY_PATH at both link time and runtime so that mir-extractor can
  # find rustc_driver and friends.
  enterShell = ''
    export LD_LIBRARY_PATH="$(rustc --print sysroot)/lib:''${LD_LIBRARY_PATH:-}"
  '';
}
