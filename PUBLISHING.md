# Publishing new versions of uefi-rs to Crates.io

This guide documents the best practices for publishing new versions of
the crates in this repository to [crates.io](https://crates.io/).

**It is mostly intended for maintainers of the uefi-rs project.**

## Bumping the crate versions

For ensuring compatibility within the crates ecosystem,
Cargo [recommends][cargo-semver] maintainers to follow the [semantic versioning][semver] guidelines.

This means that before publishing the changes, we need to decide
which crates were modified and how should their version numbers be incremented.

Incrementing the version number of a crate is as simple as editing
the corresponding `Cargo.toml` file and updating the `version = ...` line,
then commiting the change (preferrably on a new branch, so that all of the version bumps
can be combined in a single pull request).

### Crate dependencies

The dependency graph of the published crates in this repo is:

- `uefi-services` depends on `uefi` (the root project)
- `uefi` depends on `uefi-macros`

If there are breaking changes happening in the project, we should first publish
a new version of `uefi-macros`, then of `uefi`, then of `uefi-services` and so on.

For example, if the signature of a widely-used macro from `uefi-macros` is changed,
a new major version of that crate will have to be published, then a new version of
`uefi` (major if the previous bump caused changes in the public API of this crate as well),
then possibly a new version of `uefi-services`.

Furthermore, `uefi-macros` has the `uefi` crate as a `dev-dependency`,
and that will have to be updated in tandem with the major versions of the core crate.

### Updating the dependent crates

Remember that if a new major version of a crate gets released, when bumping the version
of it's dependents you will have to also change the dependency line for it.

For example, if `uefi-macros` gets bumped from `1.1.0` to `2.0.0`,
you will also have to update the corresponding `Cargo.toml` of `uefi` to be:

```toml
uefi-macros = "2.0.0"
```

The dependencies in `template/Cargo.toml` should also be updated to the new version.

[cargo-semver]: https://doc.rust-lang.org/cargo/reference/semver.html
[semver]: https://semver.org/

## Publishing new versions of the crates

This section is mostly a summary of the official [guide to publishing on crates.io][cargo-publishing-reference],
with a few remarks regarding the specific of this project.

Start by following the steps in the guide. When running `cargo publish`,
you will have to use a custom `--target` flag to be able to build/verify the crates:

```
cargo publish --target x86_64-unknown-uefi
```

[cargo-publishing-reference]: https://doc.rust-lang.org/cargo/reference/publishing.html

## Updating the changelog

After bumping the crate versions, we should also update the [`CHANGELOG.md`](CHANGELOG.md) file
in order to move all of the unpublished changes to their respective version, and prepare it for
tracking further changes.
