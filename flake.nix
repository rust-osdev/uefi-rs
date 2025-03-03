{
  description = "uefi-rs";

  inputs = {
    # We follow the latest stable release of nixpkgs
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    inputs@{ self, nixpkgs, ... }:
    let
      # Systems definition for dev shells and exported packages,
      # independent of the NixOS configurations and modules defined here. We
      # just use "every system" here to not restrict any user. However, it
      # likely happens that certain packages don't build for/under certain
      # systems.
      systems = nixpkgs.lib.systems.flakeExposed;
      forAllSystems =
        function: nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});

      # We directly instantiate the functionality, without using an
      # nixpkgs overlay.
      # https://github.com/oxalica/rust-overlay/blob/f4d5a693c18b389f0d58f55b6f7be6ef85af186f/docs/reference.md?plain=1#L26
      rustToolchain =
        pkgs:
        let
          rust-bin = (inputs.rust-overlay.lib.mkRustBin { }) pkgs;
          rustToolchainBuilder = import ./nix/rust-toolchain.nix;
        in
        rustToolchainBuilder { inherit rust-bin; };
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            nixfmt-rfc-style

            # Integration test dependencies
            swtpm
            qemu

            # Rust toolchain
            (rustToolchain pkgs)

            # Other
            cargo-llvm-cov
            mdbook
            yamlfmt
            which # used by "cargo xtask fmt"
          ];

          # Set ENV vars.
          # OVMF_CODE="${pkgs.OVMF.firmware}";
          # OVMF_VARS="${pkgs.OVMF.variables}";
          # OVMF_SHELL="${pkgs.edk2-uefi-shell}";
        };
      });
      formatter = forAllSystems (pkgs: pkgs.nixfmt-rfc-style);
    };
}
