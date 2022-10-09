## 0.7.1-rc.19

### Features

- python bindings (#145)

## 0.7.1-rc.18

### Features

- python bindings (#145)

## 0.7.1-rc.17

### Features

- python bindings (#145)

## 0.7.1-rc.16

### Features

- python bindings (#145)

## 0.7.1-rc.15

### Features

- python bindings (#145)

## 0.7.1-rc.14

### Features

- python bindings (#145)

## 0.7.1-rc.13

### Features

- python bindings (#145)

## 0.7.1-rc.12

### Features

- python bindings (#145)

## 0.7.1-rc.11

### Features

- python bindings (#145)

## 0.7.1-rc.10

### Features

- python bindings (#145)

## 0.7.1-rc.9

### Features

- python bindings (#145)

## 0.7.1-rc.8

### Features

- python bindings (#145)

## 0.7.1-rc.7

### Features

- python bindings (#145)

## 0.7.1-rc.6

### Features

- python bindings (#145)

## 0.7.1-rc.5

### Features

- python bindings (#145)

## 0.7.1-rc.4

### Features

- python bindings (#145)

## 0.7.1-rc.3

### Features

- python bindings (#145)

## 0.7.1-rc.2

### Features

- python bindings (#145)

## 0.7.1-rc.1

### Features

- python bindings (#145)

## 0.7.1-rc.0

### Features

- python bindings (#145)

## 0.7.0-rc.0

### Breaking Changes

- added `#[must_use]` to `JobHandle::job_id` at clippy's
suggestion.
- Regenerate API client with latest generator (#114)
- The return type of `Executable::execute_on_qpu` and `Executable::execute_on_qvm` is a new `ExecutionData` struct. The previous result map is now stored on `ExecutionData::registers` with the renamed `RegisterData` type (previously `ExecutionResult`).
- Changed the error types returned from all public methods.
- Return the same error type from both qvm and qpu execution. (#55)
- Switch to builder-interface for execution to enable parametric compilation. (#11)
- Refactor C lib to use tagged union responses, add C book. (#8)
- Add qpu C interface and refactor into fewer crates with common interfaces.
- Change CLI to allow running Quil on targets from files. Add missing metadata and some logging.
- Further error simplification and integration tests for `qpu`
- Make config paths configurable via environment variables and expose config errors.
- Simplify QVM interface to match what QPU will be. Remove C interface to list QPUs.

### Features

- python bindings (#145)
- export JobHandle and derive traits on it (#129)
- derive more traits on types (#125)
- Add a way to split job submission and result fetching. (#113)
- Return billable QPU execution time in results. (#90)
- Replace `eyre` with `thiserror` for all errors. (#63)
- Add `compile_with_quilc` option on `Executable` to support Quil-T. (#57)
- Add Lodgepole integration to run on real QPUs!
- Add translation service `qpu` call and token refresh capabilities.
- New `quilc` crate for communicating with quilc from Rust.
- Load QVM/QCS URLs from config.
- Basic OpenAPI C-bindings
- Allow arbitrary output register names.
- Added ability to run a basic program on QVM through C with lots of limitations.

### Fixes

- Stop blocking the async runtime when connecting to quilc or a QPU. (#61)
- Sync RZ fidelity with pyQuil implementation. (#60)
- Export Error type
- Make generated macOS .dylib files portable.

## 0.6.0

### Breaking Changes

- added `#[must_use]` to `JobHandle::job_id` at clippy's
suggestion.

### Features

- export JobHandle and derive traits on it (#129)

## 0.5.0

### Breaking Changes

- added `#[must_use]` to `JobHandle::job_id` at clippy's
suggestion.

### Features

- export JobHandle and derive traits on it (#129)

## 0.4.0

### Breaking Changes

- Regenerate API client with latest generator (#114)

### Features

- derive more traits on types (#125)

## [0.3.3](https://github.com/rigetti/qcs-sdk-rust/compare/v0.3.2...v0.3.3) (2022-07-28)

### Features

- Added `Executable::submit_to_qpu` and `Executable::retrieve_results` methods to allow splitting program submission and result retrieval into separate steps. This also enables logging of the QCS job ID for easier debugging.

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
