# How to contribute to uefi-rs

Pull requests, issues and suggestions are welcome!

## Workflow

First, change to the `uefi-test-runner` directory:

```shell
cd 'uefi-test-runner'
```

Make some changes in your favourite editor / IDE:
I use [VS Code][code] with the [RLS][rls] extension.

Test your changes:

```shell
./build.py build run
```

[code]: https://code.visualstudio.com/
[rls]: https://github.com/rust-lang-nursery/rls-vscode

## Style guide

This repository follows Rust's [standard style][style], the same one imposed by `rustfmt`.

[style]: https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/guide/guide.md
