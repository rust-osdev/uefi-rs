let
  pkgsSrc = import ./nix/nixpkgs.nix;
  pkgs = import pkgsSrc {};
in
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
    rustup
    qemu
  ];

  buildInputs = with pkgs; [
  ];

  # Set ENV vars.
  # These are automatically the right files for the current CPU (if available).
  # https://github.com/NixOS/nixpkgs/blob/nixos-22.11/pkgs/applications/virtualization/OVMF/default.nix#L80
  OVMF_CODE="${pkgs.OVMF.firmware}";
  OVMF_VARS="${pkgs.OVMF.variables}";
}
