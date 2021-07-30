The API boundary for doing QCS-related things from C (or a C-compatible language).

## Setup

This library is implemented in Rust, so the first thing you need is [rustup]. The default, stable toolchain will work just fine.

Next, because this library relies on [ØMQ], you'll need [`cmake`] installed:

- macOS [Homebrew] : `brew install cmake`
- Windows [Chocolatey]: `choco install cmake`
- Debian: `apt install cmake`

Finally, this project uses [cargo-make] in order to orchestrate build tasks, so install that using `cargo install cargo-make`.


## Development

1. Run `makers` (no args, default flow) to build and run tests.
1. `makers lint` does linting.
1. `makers release-flow` will do a release build of the C-SDK and spit out a `.dylib` file in the current directory.

[rustup]: https://rustup.rs/
[ØMQ]: https://zeromq.org/
[cmake]: https://cmake.org/
[homebrew]: https://brew.sh/
[Chocolatey]: https://chocolatey.org/
[cargo-make]: https://sagiegurari.github.io/cargo-make/
