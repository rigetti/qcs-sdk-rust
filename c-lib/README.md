The API boundary for doing QCS-related things from C (or a C-compatible language).

## Development

 Here are some useful cargo-make tasks for this crate only (run from this directory):

2. `makers test-custom` will do a `build-debug` and then use `clang` to compile the `.c` files (linking them against the generated `.dylib`) in `tests` and run them. Exit code 0 means success! Note that the name of this task is such that running `makers test-flow` will include these tests with any Cargo tests, even if run at the workspace level.
3. `makers detect-leaks` will output the total amount of memory leaked from the test files. Currently Tokio will always leak 2600 bytes, so that's expected. Anything more than 2600 indicates a problem with manual memory management.