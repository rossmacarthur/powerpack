# ⚡ powerpack

[![Crates.io Version](https://img.shields.io/crates/v/powerpack.svg)](https://crates.io/crates/powerpack)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/powerpack)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/powerpack/build/trunk)](https://github.com/rossmacarthur/powerpack/actions?query=workflow%3Abuild)

Supercharge your [Alfred 🎩][alfred] workflows by building them in Rust 🦀!

[alfred]: https://www.alfredapp.com

## 🚀 Getting started

This project contains a `powerpack` crate which provides types for developing
script filter Alfred workflows in Rust. It also provides a command line tool to
initialize, build, and install  workflows built using the `powerpack` crate.

Firstly, install the command line tool.
```sh
cargo install powerpack-cli
```

Now create a new project using a similar API as `cargo new` or `cargo init`.
```sh
powerpack new myworkflow && cd myworkflow
```

This will create a new Rust project as well as a `workflow/` directory
containing information about your Alfred workflow. The following will create
a release build of the workflow and copy it to the `workflow/` directory.
```sh
powerpack build --release
```

Now you can link it to Alfred. The following will symlink the `workflow/`
directory to the Alfred preferences folder.
```sh
powerpack link
```

Now you can run the workflow from Alfred ✨!

To package a `.alfredworkflow` file for release you can run the following.
```sh
powerpack package
```

The release will be available at `target/workflow/myworkflow.alfredworkflow`.

## 👷 GitHub Action

[`setup-crate`][setup] can be used to install `powerpack` in a GitHub Actions
workflow. For example:
```yaml
steps:
  - uses: actions/checkout@v2
  - uses: extractions/setup-crate@v1
    with:
      owner: rossmacarthur
      name: powerpack
  - run: powerpack package
  # produces an artifact at `target/workflow/{name}.alfredworkflow`
```

[setup]: https://github.com/extractions/setup-powerpack

## 💡 Examples

The following projects are built using `powerpack`.

- [crates.alfredworkflow](https://github.com/rossmacarthur/crates.alfredworkflow)
- [emojis.alfredworkflow](https://github.com/rossmacarthur/emojis.alfredworkflow)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
