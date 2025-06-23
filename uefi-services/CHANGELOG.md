# uefi-services - 0.26 (2025-06-23)

## Changed
- The deprecation warning was replaced with a `compile_error!` call to alert
  users to upgrade. `v0.25.0` can cause problems when used with `uefi` `>v0.25`.
