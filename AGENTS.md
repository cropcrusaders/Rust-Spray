# Guidelines for Codex Agents

This repository uses Rust and Docker-based cross compilation to target
embedded Linux boards like the Raspberry Pi.

## Code Style
- Run `cargo fmt` before committing.
- Document public functions using Rustdoc comments.
- Keep functions small and focused.

## Testing
- Execute `cargo test` for the host architecture before opening a pull request.

## Cross Compilation
To produce Raspberry Pi binaries the project relies on the
[cross](https://github.com/cross-rs/cross) tool and Docker images
specified in `Cross.toml`.

### Installing the `cross` CLI
The crate is no longer published on crates.io. Install it directly from
GitHub:

```bash
cargo install --git https://github.com/cross-rs/cross cross --locked
```

Docker must be available to the current user. The images referenced in
`Cross.toml` should be downloaded ahead of time if working offline.

### Building for `armv7-unknown-linux-gnueabihf`
Run the following command:

```bash
cross build --release --target armv7-unknown-linux-gnueabihf --verbose
```

The resulting binary will be located in
`target/armv7-unknown-linux-gnueabihf/release/`.
