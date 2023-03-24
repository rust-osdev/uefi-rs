# Pinned nixpkgs version.

let
  # Picked a recent commit from the nixos-22.11-small branch.
  # https://github.com/NixOS/nixpkgs/tree/nixos-22.11-small
  #
  # When you change this, also change the sha256 hash!
  rev = "a45745ac9e4e1eb86397ab22e2a8823120ab9a4c";
in
builtins.fetchTarball {
  url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  sha256 = "sha256:1acllp8yxp1rwncxsxnxl9cwkm97wxfnd6ryclmvll3sa39j9b1z";
}
