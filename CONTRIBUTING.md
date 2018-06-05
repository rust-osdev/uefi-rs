# How to contribute to uefi-rs

Pull requests, issues and suggestions are welcome!

The UEFI spec is huge, so there might be some omissions or some missing features.
You should follow the existing project structure when adding new items.

## Workflow

First, change to the `uefi-test-runner` directory:

```shell
cd 'uefi-test-runner'
```

Make some changes in your favourite editor / IDE:
I use [VS Code][code] with the [RLS][rls] extension.

Test your changes:

```shell
./build.py run
```

The line above will open a QEMU window where the test harness will run some tests.

[code]: https://code.visualstudio.com/
[rls]: https://github.com/rust-lang-nursery/rls-vscode

## Style guide

This repository follows Rust's [standard style][style], the same one imposed by `rustfmt`.

[style]: https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/guide/guide.md
