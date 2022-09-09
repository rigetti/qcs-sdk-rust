# qcs-sdk-rust

The `qcs` crate is a high-level interface to Rigetti's [Quantum Cloud Services], allowing Rust developers to run [Quil] programs on Rigetti's [QPUs]. This crate is a Rust port of [`pyQuil`], though it currently has a much smaller feature set.

The `qcs-sdk-python` crate exposes some functionality of the `qcs` crate to Python using [pyo3](https://pyo3.rs/v0.17.1/).

> For the C-bindings to this library, check out [qcs-sdk-c](https://github.com/rigetti/qcs-sdk-c)

## Documentation

This crate is documented primarily via [rustdoc] comments and examples, which are available on [docs.rs].

## Development

Most development tasks are automated with [cargo-make] (like make, but you can have dependencies on other Rust tools and a _ton_ of useful tasks are built in). Install cargo-make by doing `cargo install cargo-make`. Then you can invoke it with either `cargo make <task>` or `makers <task>`. Tasks are defined in files called `Makefile.toml`.

In order to run all checks exactly the same way that CI does, use `makers ci-flow` from the project root (workspace).


### Commits

Commits should follow the conventional commit syntax, with one of the following [scopes][scopes]:

- `lib` or `rust`: changes to the rust SDK
- `python`: changes to the Python bindings
- No scope: changes to both crates

### Dependencies

Because this library relies on [ØMQ], [`cmake`] is required:

- macOS [Homebrew] : `brew install cmake`
- Windows [Chocolatey]: `choco install cmake`
- Debian: `apt install cmake`

### Running Tests

The best way to go about this is via `makers` or `cargo make` with no task. This will default to `dev-test-flow` which formats all code, builds, and tests everything.

Any tests which cannot be run in CI should be run with `makers manual`. These tests require configured QCS credentials with access to internal functions, as well as a connection to the Rigetti VPN.

### Linting

`makers lint` will lint run all static checks.

### Documentation

To build the docs.rs-style docs, run `makers docs`. You can also do `makers serve-docs` to launch a local webserver for viewing immediately.

## Release

To release the library crate or the bindings to Python, manually run the `release` or `release python` workflow in GitHub Actions, respectively.

[cargo-make]: https://sagiegurari.github.io/cargo-make/
[Quantum Cloud Services]: https://docs.rigetti.com/qcs/
[Quil]: https://github.com/quil-lang/quil
[QPUs]: https://qcs.rigetti.com/qpus/
[`pyQuil`]: https://github.com/rigetti/pyquil
[rustdoc]: https://doc.rust-lang.org/rustdoc/index.html
[docs.rs]: https://docs.rs/qcs
[scopes]: https://www.conventionalcommits.org/en/v1.0.0/#commit-message-with-scope
