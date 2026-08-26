# QCS Rust SDK

This repo provides two crates:

- `qcs` which is the Rust SDK for executing quantum programs on Rigetti QPUs; and
- `qcs-sdk-python` which defines, generates, and publishes the Python bindgins
  to make use of the Rust SDK.

## Documentation

This crate is documented primarily via [rustdoc] comments and examples, which are available on [docs.rs].

## Development

Most development tasks are automated with [cargo-make] (like make, but you can have dependencies on other Rust tools and a _ton_ of useful tasks are built in). Install cargo-make by doing `cargo install cargo-make`. Then you can invoke it with either `cargo make <task>` or `makers <task>`. Tasks are defined in files called `Makefile.toml`.

In order to run all checks exactly the same way that CI does, use `makers ci-flow` from the project root (workspace).


### Commits

Commits should follow the conventional commit syntax, with one of the following [scopes](scopes):

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

### `libquil`

[`libquil`](https://github.com/rigetti/libquil) provides [quilc](https://github.com/quil-lang/quilc) and [QVM](https://github.com/quil-lang/qvm) as a shared library, which can be used by `qcs-sdk-rust` as an alternative client for those tools.

To use `libquil`:
* install the library with [libquil's own
  installer](https://github.com/rigetti/libquil#automated-installation), which
  `--install-deps` tells to install the prerequisite packages too:

  ```
  curl -fsSL "https://raw.githubusercontent.com/rigetti/libquil/v${LIBQUIL_VERSION}/scripts/install.sh" \
    | sudo bash -s -- --install-deps "${LIBQUIL_VERSION}"
  ```

* also install `libclang`, which bindgen needs to generate `libquil-sys`'s bindings.
  It ships with the Xcode command line tools on macOS; on Debian it is `libclang-dev`
* enable the feature with `--features libquil`

`--install-deps` uses `apt` on Linux and Homebrew on macOS, and stops if neither is
present, pointing at [libquil's
requirements](https://github.com/rigetti/libquil#requirements) so you can install them
with your own package manager. libquil publishes no Windows build.

<!-- TODO(github.com/rigetti/libquil#58): once the sbcl-librarian runtime work is
released as a stable version, drop LIBQUIL_VERSION from the commands above and from
CI, and install the latest release instead. -->

`libquil-sys` 0.5 links against the sbcl-librarian runtime that libquil is built on, so
it needs a libquil that installs that runtime and its headers (`sbcl_librarian.h`)
alongside `libquil.h`. Releases up to and including 0.3.2 ship neither, and building
against one of those fails in `libquil-sys`'s build script. That runtime is so far
released in libquil 0.4.0, which the commands above pin with `LIBQUIL_VERSION`; set it
to whichever build you need.

To build against a libquil source tree instead of an installed one, set
`LIBQUIL_SRC_PATH` to that directory; `libquil-sys` searches it before the system
locations.

### Linting

`makers lint` will lint run all static checks.

### Documentation

To build the docs.rs-style docs, run `makers docs`. You can also do `makers serve-docs` to launch a local webserver for viewing immediately.

## Release

To release, manually run the `Prepare Release` workflow in GitHub Actions.

Pre-releases for both the library crate and Python package happen automatically on merge to main.

## CI

This repository uses GitHub actions for its CI. If you are making changes to a workflow, consider using our [test events](.github/test_events/README.md) to help validate the changes.

[cargo-make]: https://sagiegurari.github.io/cargo-make/
[Quantum Cloud Services]: https://docs.rigetti.com/qcs/
[Quil]: https://github.com/quil-lang/quil
[QPUs]: https://qcs.rigetti.com/qpus/
[`pyQuil`]: https://github.com/rigetti/pyquil
[rustdoc]: https://doc.rust-lang.org/rustdoc/index.html
[docs.rs]: https://docs.rs/qcs
[scopes]: https://www.conventionalcommits.org/en/v1.0.0/#commit-message-with-scope
