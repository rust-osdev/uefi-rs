# Publishing new versions of uefi-rs to Crates.io

This guide documents how to publish new versions of the crates in this
repository to [crates.io](https://crates.io/).

**It is mostly intended for maintainers of the uefi-rs project.**

## Goal: What We Want
- We want precise entries in the [`CHANGELOG.md`] for every release that
  **follow our existing format**. Changes are supposed to be added before a
  release is performed.
- We want git tags like `uefi-raw-v0.4.0` or `uefi-v0.25.0` for every release.
- We want our crate dependencies published in the right order (if necessary):
  - `uefi` depends on `uefi-macros` and `uefi-raw`

## How: Ways to Publish

There are two major ways for releasing/publishing.

### GitHub CI Workflow (**recommended**)

1. Create a branch that updates the versions of all packages you want to
   release. This branch should include all related changes such as updating the
   versions in dependent crates and updating the changelog.
2. Create a PR of that branch. The subject of the PR must start with `release:`,
   the rest of the message is arbitrary. [Example](https://github.com/rust-osdev/uefi-rs/pull/1001)
3. Once the PR is approved and merged, a GitHub Actions workflow will take care
   of creating git tags and publishing to crates.io.

#### Crates.io secret token

The `release.yml` workflow expects a repository secret named
`CARGO_REGISTRY_TOKEN`. This is set in the [repository settings][secret]. The
value must be a crates.io [API token]. The scope of the token should be
restricted to `publish-update`.

[secret]: https://github.com/rust-osdev/uefi-rs/settings/secrets/actions
[API token]: https://crates.io/settings/tokens

### Manual

To simplify things, you can use the [`cargo-release`](https://crates.io/crates/cargo-release)
utility, which automatically bumps crate versions in `Cargo.toml`, creates
git tags as we want them, and creates a release commit. If you prefer a more
manual process, create the tags manually so that it matches the existing git tag
scheme.

*The following guide assumes that you are using `cargo-release`.*

1. Make sure that the `main` branch passes CI. Also verify that locally by
   running `cargo xtask test && cargo xtask run`.
2. Create an issue similar to <https://github.com/rust-osdev/uefi-rs/issues/955>
   for noting what you want to release.
3. Make sure that the [`CHANGELOG.md`] is up-to-date and matches the format.
4. Perform a dry run: `cargo release -p uefi-raw 0.4.0`
   Hint: `cargo-release` will automatically increase the version number for you,
   if it is still lower.
5. Release:  `cargo release -p uefi-raw 0.4.0 --execute`
6. Update the lock file: `cargo xtask build`
7. If necessary: Bump the dependency in the dependent crates, commit this, and
   if applicable release them. Go back to `4.` for that.
8. Search the repository for outdated versions, such as in
   `template/Cargo.toml`. Run `cargo xtask build` again and update + commit the
   lock file, if there are changes.
9. Submit a PR. Make sure to push all tags using `git push --tags`.
10. Update this document, in case something is inconvenient or unclear.

## General Guidelines and Tips

### Publishing Principals for crates.io

Make sure to be familiar with the [general publishing principals][cargo-publishing-reference]
for [crates.io](https://crates.io).

[cargo-publishing-reference]: https://doc.rust-lang.org/cargo/reference/publishing.html

### Details of the release pull request

For ensuring compatibility within the crates ecosystem,
Cargo [recommends][cargo-semver] maintainers to follow the [semantic versioning][semver] guidelines.

This means that before publishing the changes, we need to decide
which crates were modified and how should their version numbers be incremented.

***Note** that `0.x` -> `0.(x+1)` is allowed to be a breaking change by Cargo.*

[cargo-semver]: https://doc.rust-lang.org/cargo/reference/semver.html
[semver]: https://semver.org/

[`CHANGELOG.md`]: CHANGELOG.md
