# qcs-sdk-rust

A work in progress implementation of pyQuil-like features in Rust.

> For the C-bindings to this library, check out [qcs-sdk-c](https://github.com/rigetti/qcs-sdk-c)

## Crates

### Public

These are the top level crates intended for use by third parties.

- [qcs]: For running programs on a QPU or QVM from Rust.

### Internal

These are auxiliary crates not intended for use outside of development.

- [qcs-api](./qcs-api/README.md): A generated OpenAPI client for QCS.
- [cli](./cli/README.md): A toy CLI for testing QCS things during development.

## Development

Most development tasks are automated with [cargo-make] (like make, but you can have dependencies on other Rust tools and a _ton_ of useful tasks are built in). Install cargo-make by doing `cargo install cargo-make`. Then you can invoke it with either `cargo make <task>` or `makers <task>`. Tasks are defined in files called `Makefile.toml`. If a task is defined in the top level (workspace) file with `workspace = False` then it will only be run once. Otherwise, cargo-make will attempt to run that command for each crate.

In order to run all checks exactly the same way that CI does, use `makers workspace-ci-flow` from the project root (workspace).

### Running Tests

The best way to go about this is via `makers` or `cargo make` with no task. This will default to `dev-test-flow` which formats all code, builds, and tests everything.

Any tests which cannot be run in CI should be run with `makers manual`. These tests require configured QCS credentials with access to internal functions, as well as a connection to the Rigetti VPN.

### Linting

`makers lint` from the workspace level will lint all crates except generated ones (where `#![allow(clippy::all)]` should be included).

For new crates, the following code block should be added to the top of the `main.rs` or `lib.rs`:

```rust
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]
```

### 

### Documentation

To build the docs.rs-style docs, run `makers docs`. Only the [qcs] crate will have published docs in this format, so it's usually not worth running this at the workspace level. From within the [qcs] crate you can also do `makers serve-docs` to launch a local webserver for viewing immediately.

## Release

Before release, `makers manual` must be run in order to run tests against live QCS/QPUs.

[cargo-make]: https://sagiegurari.github.io/cargo-make/
[qcs]: ./qcs/README.md