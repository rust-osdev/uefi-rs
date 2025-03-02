# Returns a Rust toolchain for Nix that matches the one from the toolchain file.

{
  # Comes from rust-overlay
  rust-bin,
}:

# Includes rustc, cargo, rustfmt, etc
rust-bin.fromRustupToolchainFile ../rust-toolchain.toml
