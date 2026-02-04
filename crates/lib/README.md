# qcs-sdk-rust

The `qcs` crate is a high-level interface to Rigetti's [Quantum Cloud Services],
allowing Rust developers to run [Quil] programs on Rigetti's [QPUs].
This crate is a Rust port of [`PyQuil`], though it currently has a much smaller feature set.

> For the C-bindings to this library, check out [qcs-sdk-c](https://github.com/rigetti/qcs-sdk-c)

## Documentation

This crate is documented primarily via [rustdoc] comments and examples, which are available on [docs.rs].

## Features

The crate supports several features, and some of them imply others, forming a DAG:

| Feature                   | Description                                                           | Implied Features |
| ------------------------- | --------------------------------------------------------------------- | ---------------- |
| `manual-tests`            | Used to run tests that require some manual setup.                     |                  |
| `experimental`            | Enables experimental features specific to Rigetti QPUs.               |                  |
| `tracing`                 | Enables the `tracing` features in `qcs-api-client-*` crates.          |                  |
| `tracing-config`          | Enables the `tracing-config` features in `qcs-api-client-*` crates.   | `tracing`        |
| `tracing-opentelemetry`   | Enables tracing with OpenTelemetry.                                   | `tracing-config` |
| `python`                  | Exposes Python bindings via PyO3.               | `experimental`, `tracing-opentelemtry` |
| `stubs`                   | Enables Python typing for stub generation.                            | `python`         |

In addition, there currently exists the feature `_insecure-issuer-validation`
which enables a feature of the same name in the `qcs-api-client-common` crate.
The purpose of the feature there, as here, is to relax token issuer validation during testing.
It is not intended to be used in production, and not considered part of the public API.

## Changes Post Crate Merge

Prior to `v0.21` (for Python users) and `v0.25` (for Rust users),
this repository maintained separate a crate to expose Python bindings to this library.
Starting at the above versions, we've merged those crate together,
supported by the `python` feature flag.
With this change, we've united the version numbers for packages.

The new version comes with breaking changes for Python users,
which are summarized below and detailed in the `CHANGELOG`.
For the trouble, we now support the latest versions of `PyO3` and Python.
This comes with improvements to security, stability, and performance
and makes our codebase much easier to support moving forward.

### Rust Breaking Changes

For Rust consumers,
the previously `Unit`-variants of the `qvm::api::AddressRequest` `enum`
are now empty tuple variants to make them compatible with `PyO3`'s "Complex enums".
That means the following variants must have `()` appended to their usages:

```rust
// Previously you could use this:
match address_request {
    qvm::api::AddressRequest::IncludeAll => ...,
    qvm::api::AddressRequest::ExcludeAll => ...,
    qvm::api::AddressRequest::Indices(indices) => ...,
}

// Now you must use this:
match address_request {
    qvm::api::AddressRequest::IncludeAll() => ...,
    qvm::api::AddressRequest::ExcludeAll() => ...,
    qvm::api::AddressRequest::Indices(indices) => ...,
}
```

### Python Breaking Changes

For Python consumers, please be aware of the following changes:

- We are dropping support for Python 3.9 and adding support through Python 3.13.
- We no longer wrap Rust types with an additional layer for Python interop,
    but instead directly expose them as `#[pyclass]`es; in particular,
    the `from_*`, `as_*`, `to_*`, `is_*`, and `inner` methods have been removed,
    You should replace their usage with more typical Python operations (see below for examples).
- `Service.Quilc` has been renamed `Service.QUILC` to match other Python enumeration variants.
- `ExecutionData` now requires `result_data`, whereas it had been optional for `pickle` support.
    Constructing it without explicit `result_data` is almost certainly a bug,
    so now that it implements `__getnewargs__`, the `result_data` parameter is required.

#### Specific Python Examples

Instead of using `from_*`, just use the target class's constructor directly.
For example:

```diff python
- readout_value = ReadoutValues.from_integer([0, 1])
+ readout_value = ReadoutValues.Integer([0, 1])
```

Replace `is_*`, `to_*`, `as_*`, and `inner` with `match`.
Here's an example of extracting `inner` elements:

```python
match readout_value:
    case (ReadoutValues.Integer(matrix) | ReadoutValues.Real(matrix) | ReadoutValues.Complex(matrix)):
        return matrix[0]
    case _:
        raise ValueError(f"Unsupported readout type: {type(ro)}")
```

If needed, you can replace `inner` with `_0`, usually paired with an `isinstance` check.
Keep in mind that enumerated subclasses are often named after the class they take as a parameter.
The following `assert`s are all valid:

```python
readout_value = ReadoutValues.Integer([0, 1])
assert isinstance(readout_value, ReadoutValues)
assert isinstance(readout_value, ReadoutValues.Integer)
assert readout_value._0 == np.array([0, 1], dtype=np.int32)
```

## Development

Most development tasks are automated with [cargo-make] (like make,
but you can have dependencies on other Rust tools and a _ton_ of useful tasks are built in).
Install cargo-make by doing `cargo install cargo-make`.
Then you can invoke it with either `cargo make <task>` or `makers <task>`.
Tasks are defined in files called `Makefile.toml`.

In order to run all checks exactly the same way that CI does, use `makers ci-flow` from the project root (workspace).

### Dependencies

Because this library relies on [ØMQ], [`cmake`] is required:

- macOS [Homebrew] : `brew install cmake`
- Windows [Chocolatey]: `choco install cmake`
- Debian: `apt install cmake`

### Running Tests

The best way to go about this is via `makers` or `cargo make` with no task.
This will default to `dev-test-flow` which formats all code, builds, and tests everything.

Any tests which cannot be run in CI should be run with `makers manual`.
These tests require configured QCS credentials with access to internal functions,
as well as a connection to the Rigetti VPN.

### Linting

`makers lint` will lint run all static checks.

### Documentation

To build the docs.rs-style docs, run `makers docs`.
You can also do `makers serve-docs` to launch a local webserver for viewing immediately.

## Release

To release this crate, manually run the `release` workflow in GitHub Actions.

[cargo-make]: https://sagiegurari.github.io/cargo-make/
[quantum cloud services]: https://docs.rigetti.com/qcs/
[quil]: https://github.com/quil-lang/quil
[qpus]: https://qcs.rigetti.com/qpus/
[`pyquil`]: https://github.com/rigetti/pyquil
[rustdoc]: https://doc.rust-lang.org/rustdoc/index.html
[docs.rs]: https://docs.rs/qcs
