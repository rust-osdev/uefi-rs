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
  # OVMF_CODE="${pkgs.OVMF.firmware}";
  # OVMF_VARS="${pkgs.OVMF.variables}";
  # OVMF_SHELL="${pkgs.edk2-uefi-shell}";
}
