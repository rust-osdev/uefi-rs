let
  sources = import ./sources.nix { };
  rust-overlay = import sources.rust-overlay;
in
import sources.nixpkgs { overlays = [ rust-overlay ]; }
