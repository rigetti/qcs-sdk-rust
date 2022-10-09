## 0.2.1-rc.19

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.18

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.17

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.16

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.15

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.14

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.13

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.12

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.11

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.10

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.9

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.8

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.7

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.6

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.5

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.4

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.3

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.2

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.1

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.0

### Features

- add type hints
- python bindings (#145)

## 0.2.0-rc.0

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

## 0.1.0

### Features
This initial RC release exposes the following functions to Python:
	- compile
	- rewrite_arithmentic
	- translate
	- submit
	- retrieve_results
	- build_patch_values

As this is a release candidate, they can be used experimentally to get a preview of upcoming functionality. Please report any issues and expect that these signatures and workflows are going to be changed before a full release.
