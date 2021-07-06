A work in progress implementation of pyQuil-like features in Rust.

## Crates

- [qcs-api](./qcs-api/README.md): A generated OpenAPI client for QCS.
- [qcs-util](./qcs-util/README.md): Manual functions for interacting with QCS API (e.g. authentication).
- [c-lib](./c-lib/README.md): The C-API binding to QCS things
- [cli](./cli/README.md): A toy CLI for testing QCS things during development.

## Development

Most development tasks are automated with cargo-make (like make, but you can have dependencies on other Rust tools and a _ton_ of useful tasks are built in). Install cargo-make by doing `cargo install cargo-make`. Then you can invoke it with either `cargo make <task>` or `makers <task>`. Tasks are defined in files called `Makefile.toml`. If a task is defined in the top level (workspace) file with `workspace = False` then it will only be run once. Otherwise, cargo-make will attempt to run that command for each crate.

### Running Tests

The best way to go about this is via `makers` or `cargo make` with no task. This will default to `dev-test-flow` which formats all code, builds, and tests everything.

### Linting

`makers clippy-flow` from the workspace level will lint all crates except generated ones (where `#![allow(clippy::all)]` should be included).

For new crates, the following code block should be added to the top of the `main.rs` or `lib.rs`, except the unsafe lint if you need unsafe code (e.g. the c-lib crate):

```rust
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]
```