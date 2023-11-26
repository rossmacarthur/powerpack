# hello-alfred

This is an example workflow built using `powerpack`.

Use the `build` command to build this example.
```sh
cargo run --package powerpack-cli -- build --package hello-alfred --release
```

Now use the `link` command to symlink to Alfred.
```sh
cargo run --package powerpack-cli -- link --package hello-alfred
```
