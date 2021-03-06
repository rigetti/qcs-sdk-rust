## [0.3.2](https://github.com/rigetti/qcs-sdk-rust/compare/v0.3.1...v0.3.2) (2022-06-18)

Maintenance release, updating dependencies only.

## [0.3.1](https://github.com/rigetti/qcs-sdk-rust/compare/v0.3.0...v0.3.1) (2022-06-15)

Maintenance release, updating dependencies only.

## [0.3.0](https://github.com/rigetti/qcs-sdk-rust/compare/v0.2.2...v0.2.3) (2022-06-04)

### Breaking Changes

- The return type of `Executable::execute_on_qpu` and `Executable::execute_on_qvm` is a new `ExecutionData` struct. The previous result map is now stored on `ExecutionData::registers` with the renamed `RegisterData` type (previously `ExecutionResult`). (#90)

### Features

- Return billable QPU execution time in results. (#90)

## [0.2.2](https://github.com/rigetti/qcs-sdk-rust/compare/v0.2.1...v0.2.2) (2022-06-02)

Maintenance release, updating dependencies only.

## [0.2.1](https://github.com/rigetti/qcs-sdk-rust/compare/v0.2.0...v0.2.1) (2022-05-31)

Maintenance release, updating dependencies only.

## [0.2.0](https://github.com/rigetti/qcs-sdk-rust/compare/v0.1.1...v0.2.0) (2022-04-05)

### Breaking Changes

- Changed the error types returned from all fallible public functions. (#63)

### Features

- Return more detailed error types from all fallible functions. (#63)

### Fixes

- Sync RZ fidelity with pyQuil implementation. (#60)
- Stop blocking the async runtime when connecting to quilc or a QPU. (#61)

## [0.1.1](https://github.com/rigetti/qcs-sdk-rust/compare/v0.1.0...v0.1.1) (2022-03-23)

### Features
* Add `compile_with_quilc` option on `Executable` to support Quil-T. by @dbanty in [#57](https://github.com/rigetti/qcs-sdk-rust/pull/57)

## [0.1.0](https://github.com/rigetti/qcs-sdk-rust/compare/v0.0.3...v0.1.0) (2022-03-18)

### Breaking Changes

* The error type of `Executable::execute_on_qpu` and `Executable::execute_on_qvm` has changed.

### Features

* You can now tell the difference between an error which warrants a retry (`Error::Retry`) and an error which is fatal (`Error::Fatal`).

## [0.0.3](https://github.com/rigetti/qcs-sdk-rust/compare/v0.0.2...v0.0.3) (2022-02-07)

### Docs

* Update qcs crate README to be more descriptive on crates.io. Move build instructions into workspace README. (#25) ([9d3af44](https://github.com/rigetti/qcs-sdk-rust/commit/9d3af44acee42a4ac03e1ff0fdc3582db985d9ce)), closes [#25](https://github.com/rigetti/qcs-sdk-rust/issues/25)

### Update

* Derive `Serialize` on `ExecutionResult` ([9bfec9b](https://github.com/rigetti/qcs-sdk-rust/commit/9bfec9bdca765f0858aa90d000a50feb436bc036))
* Log an error when there's a problem loading config for easier debugging. ([cf9e5cf](https://github.com/rigetti/qcs-sdk-rust/commit/cf9e5cf9b90487b9d0f5a5155888fa8d33e88b31))

## [0.0.2](https://github.com/rigetti/qcs-sdk-rust/compare/v0.0.1...v0.0.2) (2021-10-19)


### Upgrade

* Switch to released version of quil-rs ([df5b481](https://github.com/rigetti/qcs-sdk-rust/commit/df5b481196f0cb09da9d971613a3fe1d0d68406d))

## [0.0.1](https://github.com/rigetti/qcs-sdk-rust/compare/v0.0.0...v0.0.1) (2021-10-14)


### New

* Initial public release ([d4fcc09](https://github.com/rigetti/qcs-sdk-rust/commit/d4fcc09db8cfbc85f4545fe88f675da3c0a7e435))
