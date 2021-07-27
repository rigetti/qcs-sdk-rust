A work in progress implementation of pyQuil-like features in Rust.

## Crates

### Public

These are the top level crates intended for use by third parties.

- [qcs]: For running programs on a QPU or QVM from Rust.
- [c-lib]: The C-API bindings to `qcs` allowing consumption directly from C or other languages that speak C.

### Internal

These are auxiliary crates not intended for use outside of development.

- [qcs-api](./qcs-api/README.md): A generated OpenAPI client for QCS.
- [cli](./cli/README.md): A toy CLI for testing QCS things during development.

## Development

Most development tasks are automated with [cargo-make] (like make, but you can have dependencies on other Rust tools and a _ton_ of useful tasks are built in). Install cargo-make by doing `cargo install cargo-make`. Then you can invoke it with either `cargo make <task>` or `makers <task>`. Tasks are defined in files called `Makefile.toml`. If a task is defined in the top level (workspace) file with `workspace = False` then it will only be run once. Otherwise, cargo-make will attempt to run that command for each crate.

### Running Tests

The best way to go about this is via `makers` or `cargo make` with no task. This will default to `dev-test-flow` which formats all code, builds, and tests everything.

Any tests which cannot be run in CI should be run with `makers manual`. These tests require configured QCS credentials with access to internal functions, as well as a connection to the Rigetti VPN.

### Linting

`makers clippy-flow` from the workspace level will lint all crates except generated ones (where `#![allow(clippy::all)]` should be included).

For new crates, the following code block should be added to the top of the `main.rs` or `lib.rs`, except the unsafe lint if you need unsafe code (e.g. the c-lib crate):

```rust
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![forbid(unsafe_code)]
```

### Documentation

Docs format vary from crate to crate. `makers docs` builds the documentation for every crate. 

`makers serve-docs` builds and hosts the docs for __public crates only__. This command should be run in the directory of the crate you wish to view docs for, not at the project root.

For [qcs], this is a normal rustdoc build for the public API. For [c-lib], docs are built using [mdbook].

## Release

Before release, `makers manual` must be run in order to run tests against live QCS/QPUs.

[cargo-make]: https://sagiegurari.github.io/cargo-make/
[c-lib]: ./c-lib/README.md
[qcs]: ./qcs/README.md
[mdbook]: https://rust-lang.github.io/mdBook/