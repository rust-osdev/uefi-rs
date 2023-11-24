# Returns the Rust toolchain for Nix compliant to the rust-toolchain.toml file
# but without rustup.

{
  # Comes from rust-overlay
  rust-bin
}:

# Includes rustc, cargo, rustfmt, etc
rust-bin.fromRustupToolchainFile ../rust-toolchain.toml
