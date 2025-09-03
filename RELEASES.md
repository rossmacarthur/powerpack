# 📝 Release notes

### 0.7.0

*September 3rd, 2025*

#### powerpack-cli

- [Add `check` subcommand][a8b33f98]. Works exactly like the `build` subcommand
  except it does not copy the binary across to the workflow.

[a8b33f98]: https://github.com/rossmacarthur/powerpack/commit/a8b33f98e58433fa5d8f14918ee5309e0f84da27

#### powerpack

- [Upgrade to Rust 1.89 and edition 2024][cb6ceacf]. This change raises the
  minimum supported Rust version to 1.89. This removed some unnecessary
  dependencies.

[cb6ceacf]: https://github.com/rossmacarthur/powerpack/commit/cb6ceacf1703551678002451f485f3b12cc2b10b

### 0.6.3

*February 23rd, 2025*

#### powerpack

- [Tweak logging of errors in `-cache` and `-detach` crates.][a8e70a74]

[a8e70a74]: https://github.com/rossmacarthur/powerpack/commit/a8e70a7417723df3db2bca8b51a487408ab636b8

### 0.6.2

*February 23rd, 2025*

#### powerpack

- [Fix feature check for `powerpack::cache`.][0a745db9]

[0a745db9]: https://github.com/rossmacarthur/powerpack/commit/0a745db93f827e067b15d008171ff3c2157ef34a

### 0.6.1

*February 23rd, 2025*

#### powerpack

- [Add cache crate.][2c3ca62a] Provides a cache management system for Alfred
  workflows. This crate is gated behind the `cache` feature in `powerpack`. Data
  is stored in the Alfred workflow cache directory by default. The cache is
  updated asynchronously using `powerpack::detach::spawn`.

  See the crate level documentation for more information.

- [Add logger crate.][029f48d8] Adds a simple file logger implementation that
  logs to the Alfred workflow cache directory by default. This crate is gated
  behind the `logger` feature in `powerpack`.

  ```rust
  use powerpack::logger;

  logger::Builder::new()
      .filename(
        concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION"), ".log")
      )
      .max_level(logger::LevelFilter::Warn)
      .init();
  ```

  See the crate level documentation for more information.

- [env: Add defaulting workflow cache and data fns.][e28d8216] Add two new
  functions that fetch the workflow cache and data directories and instead of
  returning an `Option` return a sensible default value if the Alfred
  environment variables are not set.

  ```rust
  use powerpack::env;

  let cache_dir = env::workflow_cache_or_default();
  let data_dir = env::workflow_data_or_default();
  ```

  There are also fallible versions which only fail if the user's home directory
  cannot be determined;
  ```rust
  use powerpack::env;

  let cache_dir = env::try_workflow_cache_or_default()?;
  let data_dir = env::try_workflow_data_or_default()?;
  ```


[2c3ca62a]: https://github.com/rossmacarthur/powerpack/commit/2c3ca62a99a9c67ddc92169ff17a2237a0b097b7
[029f48d8]: https://github.com/rossmacarthur/powerpack/commit/029f48d84e5dcb3c2f14e859f47230d86c92f3c9
[e28d8216]: https://github.com/rossmacarthur/powerpack/commit/e28d8216333be7fbd81b159d3549e2bb798ec6d4

### 0.6.0

*January 26th, 2025*

#### powerpack

- [Add support for variables.][a61b43ce] Adds [`Output::variables`],
  [`Item::variables`], and [`Modifier::variables`] which allow you to set the
  variables which are passed out of the script filter object.

  These remain accessible throughout the current session as environment
  variables. In addition, they are passed back in when the script reruns within
  the same session. This can be used for managing state between runs as the user
  types input or when the script is set to re-run after an interval.

- [Add `compile_error!` binary to `powerpack` crate.][b8cc449c] If you try to
  install `powerpack` as a binary it will now direct you to install
  `powerpack-cli`.

[`Modifier::variables`]: https://docs.rs/powerpack/latest/powerpack/struct.Modifier.html#method.variables
[`Item::variables`]: https://docs.rs/powerpack/latest/powerpack/struct.Item.html#method.variables
[`Output::variables`]: https://docs.rs/powerpack/latest/powerpack/struct.Output.html#method.variables

[a61b43ce]: https://github.com/rossmacarthur/powerpack/commit/a61b43ced346edf1cd9e019c6dda046939433e5c
[b8cc449c]: https://github.com/rossmacarthur/powerpack/commit/b8cc449c585275285352c359c23da32412ab79a2

### 0.5.0

*December 31st, 2023*

#### powerpack-cli

- [Support workspaces with `--package` flag.][32b8bf1b] Previously `powerpack`
  only worked with the root package of a workspace. You can now have multiple
  packages in a workspace and use the `--package` flag to specify which package
  to build, link or package. The `workflow/` directory containing the package
  information must be in the same directory as the manifest file for the
  particular package.

#### powerpack

- [Support `skipknowledge` option.][b4f0f8cf] Adds [`Output::skip_knowledge`]
  which allows you to set `uid` and preserve the item order while allowing Alfred
  to retain knowledge of your items, like your current selection during a re-run.

- [Support Universal Actions.][887ae3ac] Adds [`Item::action`] which allows you
  to set the universal action(s) for an item.

- [Support multiple item arguments.][beb28208] Adds [`Item::args`] which allows
  you to set multiple arguments which will be passed as a JSON array.

- [Support multiple modifier keys.][9e8b210e] Adds [`Modifier::new_multi`] which
  allows you to specify a combination of keys as the modifier.

[`Item::action`]: https://docs.rs/powerpack/latest/powerpack/struct.Item.html#method.action
[`Item::args`]: https://docs.rs/powerpack/latest/powerpack/struct.Item.html#method.args
[`Output::skip_knowledge`]: https://docs.rs/powerpack/latest/powerpack/struct.Output.html#method.skip_knowledge
[`Modifier::new_multi`]: https://docs.rs/powerpack/latest/powerpack/struct.Modifier.html#method.new_multi
[b4f0f8cf]: https://github.com/rossmacarthur/powerpack/commit/b4f0f8cffc2f1bbbe2445892054904b39ffaa304
[887ae3ac]: https://github.com/rossmacarthur/powerpack/commit/887ae3acbef494a164e181996f80d6100c5f3a7f
[beb28208]: https://github.com/rossmacarthur/powerpack/commit/beb2820874c5405f0a1835b6db757deeb12f1d0e
[9e8b210e]: https://github.com/rossmacarthur/powerpack/commit/9e8b210e11a0b6c68bbb15be4e1a23504fbfe1f9
[32b8bf1b]: https://github.com/rossmacarthur/powerpack/commit/32b8bf1bd7126cf615c72477b2e66efbcf7c772a

---
### 0.4.2

*September 20th, 2022*

#### powerpack-cli

- [Fix for an empty syncfolder in Alfred config.][#8] We now use the default in
  this case.

- [Fix for when target dir is outside of current dir.][#9]

Thanks [@knutwalker](https://github.com/knutwalker) for these fixes!

[#8]: https://github.com/rossmacarthur/powerpack/pull/8
[#9]: https://github.com/rossmacarthur/powerpack/pull/9


#### powerpack

- [Convert `env` module into a `powerpack-env` crate.](#todo) Re-exported as
  `powerpack::env` when the `env` feature is enabled in `powerpack`. This
  feature is enabled by default.

---
### 0.4.1

*March 20th, 2022*

#### powerpack-cli

- [Fix copying under latest macOS.][38943ba]

[38943ba]: https://github.com/rossmacarthur/powerpack/commit/38943ba0f44b59052b37d1ae1815f9baf31ab068

---
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

---
### 0.3.1

*February 10th, 2022*

#### powerpack

- [Add `powerpack-detach` crate.][bfb3492] Re-exported as `powerpack::detach`
  when the `detach` feature is enabled in `powerpack`.

[bfb3492]: https://github.com/rossmacarthur/powerpack/commit/bfb34921503fee1661ab0f0f97c22cb8e4f1907c

---
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

---
### 0.2.2

*January 18th, 2022*

#### powerpack-cli

- [Support multiple binaries.][#5]
  Thanks [@danbi2990](https://github.com/danbi2990) for this feature!

[#5]: https://github.com/rossmacarthur/powerpack/pull/5

---
### 0.2.1

*September 4th, 2021*

#### powerpack

- [Support non-Unicode paths in workflow env.][852b884]

[852b884]: https://github.com/rossmacarthur/powerpack/commit/852b884f7a51d3f7746587bd4c80b31d74c6b3bb

---
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

---
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

---
### 0.1.1

*April 1st, 2021*

#### powerpack-cli

- [Add output to `init` command.][efcd708]
- [Fix bug in `main.rs` template.][70394b3]
- [Add `package` command.][6766f16]

[6766f16]: https://github.com/rossmacarthur/powerpack/commit/6766f16cf42411e13d0a08bda82bbf20b97e1abe
[70394b3]: https://github.com/rossmacarthur/powerpack/commit/70394b33f0f2773d1aba2127a389eb20590a24d5
[efcd708]: https://github.com/rossmacarthur/powerpack/commit/efcd70843d4768be6c35bcdbcc2c11b6cbce7ea0

---
### 0.1.0

*March 31st, 2021*

First version.
