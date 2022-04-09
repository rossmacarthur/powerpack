# 📝 Release notes

### 0.4.2

*Unreleased*

#### powerpack

- [Convert `env` module into a `powerpack-env` crate.](#todo) Re-exported as
  `powerpack::env` when the `env` feature is enabled in `powerpack`. This
  feature is enabled by default.

### 0.4.1

*March 20th, 2022*

#### powerpack-cli

- [Fix copying under latest macOS.][38943ba]

[38943ba]: https://github.com/rossmacarthur/powerpack/commit/38943ba0f44b59052b37d1ae1815f9baf31ab068

### 0.4.0

*March 19th, 2022*

#### powerpack

- [Support `rerun` script filter parameter.][9db2b1e]

- [Improve modifier key ergonomics.][5ef626c]. New `Modifier` type that takes
  the `Key` on construction. Example usage:

  ```rust
  use powerpack::{Modifier, Key};

  let item = Item::new("Hello World!")
      .subtitle("original subtitle")
      .modifier(
          Modifier::new(Key::Command)
              .subtitle("⌘ changes the subtitle")
      );
  ```

- [Add `.copy_text()` and `.large_type_text()` setters.][707a28f]

- [The public API no longer uses any clone-on-write types.][ce1b88f]

[5ef626c]: https://github.com/rossmacarthur/powerpack/commit/5ef626c24a3b3fbdfaf2197c72e5ef75dae4d453
[707a28f]: https://github.com/rossmacarthur/powerpack/commit/707a28f6df773a4e6469f60fca03d6c286a43851
[ce1b88f]: https://github.com/rossmacarthur/powerpack/commit/ce1b88f931b3f9002d034afd30d943fe321847e3
[9db2b1e]: https://github.com/rossmacarthur/powerpack/commit/9db2b1e05ef0cf7fd4a24de5e14dd8f68aad5f92

#### powerpack-cli

- [Add --force option to link subcommand.][b1d156d]

[b1d156d]: https://github.com/rossmacarthur/powerpack/commit/b1d156dda02f10c8bc787e6c20c62799385f4924

### 0.3.1

*February 10th, 2022*

#### powerpack

- [Add `powerpack-detach` crate.][bfb3492] Re-exported as `powerpack::detach`
  when the `detach` feature is enabled in `powerpack`.

[bfb3492]: https://github.com/rossmacarthur/powerpack/commit/bfb34921503fee1661ab0f0f97c22cb8e4f1907c

### 0.3.0

*February 6th, 2022*

#### powerpack-cli

- [Support --bin option.][06dc187] If a package has multiple binaries and you
  only want to build or package one or some of them then you can use this
  option to filter the binaries. This option can be used multiple times.
  ```sh
  powerpack package --bin my_bin --bin my_other_bin
  ```

- [Support --target option.][49eb415] This means you can now easily build and
  package workflows for both `x86_64-apple-darwin` and `aarch64-apple-darwin`
  from either host.
  ```sh
  powerpack package --target aarch64-apple-darwin
  ```

[06dc187]: https://github.com/rossmacarthur/powerpack/commit/06dc18778e33dda0c5a046bcd1651f1bfefeb929
[49eb415]: https://github.com/rossmacarthur/powerpack/commit/49eb4159c1fcce3ceba4059da2345024c2ab66ef

### 0.2.2

*January 18th, 2022*

#### powerpack-cli

- [Support multiple binaries.][#5] <small>(Contributed by
  [**@danbi2990**](https://github.com/danbi2990).)</small>

[#5]: https://github.com/rossmacarthur/powerpack/pull/5

### 0.2.1

*September 4th, 2021*

#### powerpack

- [Support non-Unicode paths in workflow env.][852b884]

[852b884]: https://github.com/rossmacarthur/powerpack/commit/852b884f7a51d3f7746587bd4c80b31d74c6b3bb

### 0.2.0

*July 5th, 2021*

#### powerpack

- [Use `dairy::Cow` instead of `beef::Cow`.][ac59078] This type supports
  clone-on-write `Path`s.
- [Re-export `PathBuf` and `String` from `dairy`.][0a19347]
- [Rename `Icon` constructors.][c3e77a5]

[ac59078]: https://github.com/rossmacarthur/powerpack/commit/ac590784b6d87d809001b90ce83882eb1c006881
[0a19347]: https://github.com/rossmacarthur/powerpack/commit/0a19347077b25d77102ed47a362c5de596edcbd5
[c3e77a5]: https://github.com/rossmacarthur/powerpack/commit/c3e77a5d1f7c1849926382f6a770fd5352ba779f

#### powerpack-cli

- [Sort workflow `info.plist` keys on `powerpack init`.][a9735d2]
- [Prompt for author on `powerpack new`.][de5a794] This is no longer
  automatically inferred by Cargo, so we can't get it from the Cargo manifest
  anymore.

[a9735d2]: https://github.com/rossmacarthur/powerpack/commit/a9735d231f76eb5a01a3922949a34a87e792bfc2
[de5a794]: https://github.com/rossmacarthur/powerpack/commit/de5a7945765b5405bf9f5aa4299259d8c4d6a429

### 0.1.2

*May 15th, 2021*

#### powerpack

- [Add functions for fetching workflow env variables.][d547e82] For example you
  can now use `powerpack::env::workflow_cache()` to fetch the Alfred workflow
  cache directory.

[d547e82]: https://github.com/rossmacarthur/powerpack/commit/d547e82d48b970a10fd8bf2443e4345a8c9799d8

#### powerpack-cli

- [Do not include `anyhow` in `main.rs` template.][b693208]

[b693208]: https://github.com/rossmacarthur/powerpack/commit/b693208e4f380d283287da0226b2b8a582730490

### 0.1.1

*April 1st, 2021*

#### powerpack-cli

- [Add output to `init` command.][efcd708]
- [Fix bug in `main.rs` template.][70394b3]
- [Add `package` command.][6766f16]

[6766f16]: https://github.com/rossmacarthur/powerpack/commit/6766f16cf42411e13d0a08bda82bbf20b97e1abe
[70394b3]: https://github.com/rossmacarthur/powerpack/commit/70394b33f0f2773d1aba2127a389eb20590a24d5
[efcd708]: https://github.com/rossmacarthur/powerpack/commit/efcd70843d4768be6c35bcdbcc2c11b6cbce7ea0

### 0.1.0

*March 31st, 2021*

First version.
