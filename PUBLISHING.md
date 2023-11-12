# Publishing new versions of uefi-rs to Crates.io

This guide documents how to publish new versions of the crates in this
repository to [crates.io](https://crates.io/).

**It is mostly intended for maintainers of the uefi-rs project.**

## Overview

1. Create a branch that updates the versions of all packages you want to
   release. This branch should include all related changes such as updating the
   versions in dependent crates and updating the changelog.
2. Create a PR of that branch. The subject of the PR must start with `release:`,
   the rest of the message is arbitrary.
3. Once the PR is approved and merged, a Github Actions workflow will take care
   of creating git tags and publishing to crates.io.

## Details of the release pull request

For ensuring compatibility within the crates ecosystem,
Cargo [recommends][cargo-semver] maintainers to follow the [semantic versioning][semver] guidelines.

This means that before publishing the changes, we need to decide
which crates were modified and how should their version numbers be incremented.

Incrementing the version number of a crate is as simple as editing
the corresponding `Cargo.toml` file and updating the `version = ...` line,
then committing the change.

### Crate dependencies

The dependency graph of the published crates in this repo is:

- `uefi-services` depends on `uefi`
- `uefi` depends on `uefi-macros` and `uefi-raw`

### Updating the dependent crates

Remember that if a new major version of a crate gets released, when bumping the version
of it's dependents you will have to also change the dependency line for it.

For example, if `uefi-macros` gets bumped from `0.5.0` to `0.6.0`,
you will also have to update the corresponding `Cargo.toml` of `uefi` to be:

```toml
uefi-macros = "0.6.0"
```

The dependencies in `template/Cargo.toml` should also be updated to the new version.

[cargo-semver]: https://doc.rust-lang.org/cargo/reference/semver.html
[semver]: https://semver.org/

### Updating the changelog

Update the [`CHANGELOG.md`](CHANGELOG.md) file in order to move all the
unpublished changes to their respective version, and prepare it for tracking
further changes. The date of the release should be included next to the section
title as done for the other releases.

## Crates.io secret token

The release.yml workflow expects a repository secret named
`CARGO_REGISTRY_TOKEN`. This is set in the [repository settings][secret]. The
value must be a crates.io [API token]. The scope of the token should be
restricted to `publish-update`.

[secret]: https://github.com/rust-osdev/uefi-rs/settings/secrets/actions
[API token]: https://crates.io/settings/tokens
