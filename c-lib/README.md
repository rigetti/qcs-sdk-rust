The API boundary for doing QCS-related things from C (or a C-compatible language).

## Development

1. Run `makers` (no args, default flow) to build and run tests. Right now, you have to have a valid QCS access token in your `~/.qcs/secrets.toml` for the C integration tests to work.
1. `makers lint` does linting.
1. `makers release-flow` will do a release build of the C-SDK and spit out a `.dylib` file in the current directory.
