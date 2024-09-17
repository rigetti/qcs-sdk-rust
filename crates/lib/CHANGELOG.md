## 0.24.1-rc.0

### Fixes

- Bump qcs-api-client-rust versions (#500)

## 0.24.0

### Breaking Changes

- Make Python bindings public (#463)

### Fixes

- ExecutionOptions::default() now respects defaults used in ExecutionOptionsBuilder::default() (#499)

## 0.24.0-rc.0

### Breaking Changes

- Make Python bindings public (#463)

### Fixes

- ExecutionOptions::default() now respects defaults used in ExecutionOptionsBuilder::default() (#499)

## 0.23.3-rc.0

### Fixes

- ExecutionOptions::default() now respects defaults used in ExecutionOptionsBuilder::default() (#499)

## 0.23.2

### Fixes

- ignore context extraction failure (#493)

## 0.23.2-rc.0

### Fixes

- ignore context extraction failure (#493)

## 0.23.1

### Features

- ExecutionOptions supports the pickle module. (#485)

## 0.23.1-rc.0

### Features

- ExecutionOptions supports the pickle module. (#485)

## 0.23.0

### Breaking Changes

- update qcs-api deps to use a >=9 version of jsonwebtoken (#483)

## 0.23.0-rc.0

### Breaking Changes

- update qcs-api deps to use a >=9 version of jsonwebtoken (#483)

## 0.22.0

### Breaking Changes

- remove all code related to arithmetic rewriting (#480)

### Features

- support opentelemetry tracing (#474)

## 0.22.0-rc.0

### Breaking Changes

- remove all code related to arithmetic rewriting (#480)

### Features

- support opentelemetry tracing (#474)

## 0.21.11-rc.0

### Features

- support opentelemetry tracing (#474)

## 0.21.10

### Features

- increase gRPC max message size from 4MB to 50MB (#468)

### Fixes

- add explicit dependency on rigetti-pyo3's "time" feature (#476)

## 0.21.10-rc.1

### Features

- increase gRPC max message size from 4MB to 50MB (#468)

### Fixes

- add explicit dependency on rigetti-pyo3's "time" feature (#476)

## 0.21.10-rc.0

### Features

- increase gRPC max message size from 4MB to 50MB (#468)

## 0.21.9

### Fixes

- Refresh JWT only when expired, not before every request (#470)

## 0.21.9-rc.0

### Fixes

- Refresh JWT only when expired, not before every request (#470)

## 0.21.8

### Features

- Update quil-rs (#464)

## 0.21.8-rc.0

### Features

- Update quil-rs (#464)

## 0.21.7

### Features

- Update quil

## 0.21.7-rc.0

### Features

- Update quil

## 0.21.6

### Features

- Update the quil-package, pulling in bug fixes and deterministic serialization (#461)

## 0.21.6-rc.0

### Features

- Update the quil-package, pulling in bug fixes and deterministic serialization (#461)

## 0.21.5

### Features

- Update quil (#459)

## 0.21.5-rc.0

### Features

- Update quil (#459)

## 0.21.4

### Features

- gRPC web support (#449)

### Fixes

- update h2, see RUSTSEC-2024-0332 (#453)

## 0.21.4-rc.1

### Features

- gRPC web support (#449)

### Fixes

- update h2, see RUSTSEC-2024-0332 (#453)

## 0.21.4-rc.0

### Fixes

- update h2, see RUSTSEC-2024-0332 (#453)

## 0.21.3

### Fixes

- Update quil-rs, fixing a regression in DELAY parsing (#450)

## 0.21.3-rc.0

### Fixes

- Update quil-rs, fixing a regression in DELAY parsing (#450)

## 0.21.2

### Features

- ExecutionData is now serializable. (#430)

## 0.21.2-rc.0

### Features

- ExecutionData is now serializable. (#430)

## 0.21.1

### Features

- Support running a program with multiple sets of patch values in a single request. (#445)
- Improve error messages returned by the RPCQ quilc client (#444)

## 0.21.1-rc.1

### Features

- Support running a program with multiple sets of patch values in a single request. (#445)
- Improve error messages returned by the RPCQ quilc client (#444)

## 0.21.1-rc.0

### Features

- Improve error messages returned by the RPCQ quilc client (#444)

## 0.21.0

### Breaking Changes

- The calibration program is now returned from `get_quilt_calibrations` as a string. The error variants have also been slightly modified.

### Features

- Automatically retry requests that fail due to transient network errors. (#443)

### Fixes

- Requests to cancel job no longer error due to missing target. (#443)

## 0.21.0-rc.1

### Breaking Changes

- The calibration program is now returned from `get_quilt_calibrations` as a string. The error variants have also been slightly modified.

### Features

- Automatically retry requests that fail due to transient network errors. (#443)

### Fixes

- Requests to cancel job no longer error due to missing target. (#443)

## 0.21.0-rc.0

### Breaking Changes

- The calibration program is now from
`get_quilt_calibrations` as a string. The error variants have also been
slightly modified.

* simplify error map

* cleanup test

* put calibration in mock calibration program

## 0.20.6

### Features

- Jobs that have yet to begin executing can be cancelled. (#438)

## 0.20.6-rc.0

### Features

- Jobs that have yet to begin executing can be cancelled. (#438)

## 0.20.5

### Features

- instrument pyo3-opentelemetry (#391)

### Fixes

- bump quil-rs versions (#435)

## 0.20.5-rc.2

### Features

- instrument pyo3-opentelemetry (#391)

### Fixes

- bump quil-rs versions (#435)

## 0.20.5-rc.1

### Features

- instrument pyo3-opentelemetry (#391)

## 0.20.5-rc.0

### Features

- instrument pyo3-opentelemetry (#391)

## 0.20.4

### Features

- Update quil-rs/quil-py to 0.23.5/0.6.5 (#424)

## 0.20.4-rc.0

### Features

- Update quil-rs/quil-py to 0.23.5/0.6.5 (#424)

## 0.20.3

### Features

- Update quil-rs/quil-py to 0.23.4/0.6.4 (#423)
- provide ExecutionOptions configurability as ApiExecutionOptions (#415)

## 0.20.3-rc.2

### Features

- Update quil-rs/quil-py to 0.23.4/0.6.4 (#423)
- provide ExecutionOptions configurability as ApiExecutionOptions (#415)

## 0.20.3-rc.1

### Features

- provide ExecutionOptions configurability as ApiExecutionOptions (#415)

## 0.20.3-rc.0

### Features

- provide ExecutionOptions configurability as ApiExecutionOptions (#415)

## 0.20.2

### Fixes

- make rpcq timeout functions public, since initializing Client is public (#398)

## 0.20.2-rc.0

### Fixes

- make rpcq timeout functions public, since initializing Client is public (#398)

## 0.20.1

### Features

- Update quil-rs/py to 0.23.3/0.6.3 (#413)

## 0.20.1-rc.0

### Features

- Update quil-rs/py to 0.23.3/0.6.3 (#413)

## 0.20.0

### Breaking Changes

- upgrade manylinux version in CI to support updated dependencies (#408)

## 0.20.0-rc.0

### Breaking Changes

- upgrade manylinux version in CI to support updated dependencies (#408)

## 0.19.0

### Breaking Changes

- expose BackendV2 options through Python (#407)

## 0.19.0-rc.0

### Breaking Changes

- expose BackendV2 options through Python (#407)

## 0.18.3

### Features

- Add memory_values to QPUResultData (#393)

## 0.18.3-rc.0

### Features

- Add memory_values to QPUResultData (#393)

## 0.18.2

### Fixes

- bump quil in pyproject.toml to match Cargo.toml (#403)

## 0.18.2-rc.0

### Fixes

- bump quil in pyproject.toml to match Cargo.toml (#403)

## 0.18.1

### Fixes

- update quil-rs to get string serialization bugfix (#399)

## 0.18.1-rc.0

### Fixes

- update quil-rs to get string serialization bugfix (#399)

## 0.18.0

### Breaking Changes

- changes some error types and adds new bounds to some
functions

### Features

- Add libquil support (#397)

## 0.18.0-rc.0

### Breaking Changes

- changes some error types and adds new bounds to some
functions

### Features

- Add libquil support (#397)

## 0.17.5

### Features

- Add memory_values to QPUResultData

## 0.17.5-rc.2

### Features

- Add memory_values to QPUResultData

## 0.17.5-rc.1

### Features

- Add memory_values to QPUResultData

## 0.17.5-rc.0

### Features

- Add memory_values to QPUResultData

## 0.17.4

### Fixes

- The quilc RPCQ client is now safe to share between threads. (#389)

## 0.17.4-rc.0

### Fixes

- The quilc RPCQ client is now safe to share between threads. (#389)

## 0.17.3

### Features

- Improve error message when a quilc connection fails. (#384)

## 0.17.3-rc.0

### Features

- Improve error message when a quilc connection fails. (#384)

## 0.17.2

### Features

- Update quil-rs (#383)

## 0.17.2-rc.0

### Features

- Update quil-rs (#383)

## 0.17.1

### Fixes

- update quil-rs to pull fixes (#381)

## 0.17.1-rc.0

### Fixes

- update quil-rs to pull fixes (#381)

## 0.17.0

### Breaking Changes

- use crate-defined translation options (#379)

### Features

- make Executable.get_qcs_client public (#380)

## 0.17.0-rc.1

### Breaking Changes

- use crate-defined translation options (#379)

### Features

- make Executable.get_qcs_client public (#380)

## 0.17.0-rc.0

### Breaking Changes

- use crate-defined translation options (#379)

## 0.16.8

### Features

- add a type alias for 'generic' gRPC connections (#378)

## 0.16.8-rc.0

### Features

- add a type alias for 'generic' gRPC connections (#378)

## 0.16.7

### Features

- provide access to the translation backend option (#373)

## 0.16.7-rc.0

### Features

- provide access to the translation backend option (#373)

## 0.16.6

### Fixes

- update quil version (#377)

## 0.16.6-rc.0

### Fixes

- update quil version (#377)

## 0.16.5

### Fixes

- update quil-rs to pull Program::get_qubits fix (#375)

## 0.16.5-rc.0

### Fixes

- update quil-rs to pull Program::get_qubits fix (#375)

## 0.16.4

### Features

- Update quil-rs (#365)

## 0.16.4-rc.1

### Features

- Update quil-rs (#365)

## 0.16.4-rc.0

### Features

- Update quil-rs (#365)

## 0.16.3

### Features

- release with new grpc-channel construction method (#364)

## 0.16.3-rc.0

### Features

- release with new grpc-channel construction method (#364)

## 0.16.2

### Features

- update quil-rs (#363)

## 0.16.2-rc.0

### Features

- update quil-rs (#363)

## 0.16.1

### Features

- Update quil-rs (#361)

## 0.16.1-rc.0

### Features

- Update quil-rs (#361)

## 0.16.0

### Breaking Changes

- much of the API code now requires an explicit client to be provided by the caller.

### Features

- Quilc/QVM clients (#353)

## 0.16.0-rc.0

### Breaking Changes

- much of the API code now requires an explicit client to be provided by the caller.

### Features

- Quilc/QVM clients (#353)

## 0.15.6

### Features

- Update quil-rs (#350)

## 0.15.6-rc.0

### Features

- Update quil-rs (#350)

## 0.15.5

### Features

- Default endpoints and gateway accessors are cached between requests (#337)

## 0.15.5-rc.0

### Features

- Default endpoints and gateway accessors are cached between requests (#337)

## 0.15.4

### Fixes

- Return an error when execution results indicate that the job failed to run. (#331)

## 0.15.4-rc.0

### Fixes

- Return an error when execution results indicate that the job failed to run. (#331)

## 0.15.3

### Features

- Update quil-rs to 0.20.0, quil-py to 0.4.0

## 0.15.3-rc.1

### Features

- Update quil-rs to 0.20.0, quil-py to 0.4.0

## 0.15.3-rc.0

### Features

- Update quil-rs to 0.20.0, quil-py to 0.4.0

## 0.15.2

### Features

- A new `diagnostics` module has been added. The `get_report` function will print a summary of key diagnostic data to aid in debugging. (#334)

## 0.15.2-rc.0

### Features

- A new `diagnostics` module has been added. The `get_report` function will print a summary of key diagnostic data to aid in debugging. (#334)

## 0.15.1

### Features

- Add timeout to `ExecutionOptions` builder (#323)

### Fixes

- The MissingQubitBenchmark error has been demoted to a warning. (#322)

## 0.15.1-rc.1

### Features

- Add timeout to `ExecutionOptions` builder (#323)

### Fixes

- The MissingQubitBenchmark error has been demoted to a warning. (#322)

## 0.15.1-rc.0

### Fixes

- The MissingQubitBenchmark error has been demoted to a warning. (#322)

## 0.15.0

### Breaking Changes

- `submit` and `retrieve_results` now take an `ExecutionOptions` parameter for configuring the request. The contained `ConnectionStrategy` option is now used instead of setting `use_gateway` on the `QCSClient`

## 0.15.0-rc.1

### Breaking Changes

- `submit` and `retrieve_results` now take an `ExecutionOptions` parameter for configuring the request. The contained `ConnectionStrategy` option is now used instead of setting `use_gateway` on the `QCSClient`

## 0.15.0-rc.0

### Breaking Changes

- `submit` and `retrieve_results` now take an `ExecutionOptions` parameter for configuring the request. The contained `ConnectionStrategy` option is now used instead of setting `use_gateway` on the `QCSClient`

## 0.14.1

### Fixes

- Increase gRPC message size limit (#318)

## 0.14.1-rc.0

### Fixes

- Increase gRPC message size limit (#318)

## 0.14.0

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- The QCS Client now fallbacks to default values when loading incomplete settings. The default `grpc_api_url` has been updated. (#314)
- TranslationOptions builder-like struct with Python bindings (#308)

## 0.14.0-rc.3

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- The QCS Client now fallbacks to default values when loading incomplete settings. The default `grpc_api_url` has been updated. (#314)
- TranslationOptions builder-like struct with Python bindings (#308)

## 0.14.0-rc.2

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- TranslationOptions builder-like struct with Python bindings (#308)

## 0.14.0-rc.1

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- TranslationOptions builder-like struct with Python bindings (#308)

## 0.14.0-rc.0

### Breaking Changes

- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- TranslationOptions builder-like struct with Python bindings (#308)

## 0.13.1

### Fixes

- Upgrade quil-rs, resolving some issues with parsing function calls and parameterized DefGate

## 0.13.1-rc.0

### Fixes

- Upgrade quil-rs, resolving some issues with parsing function calls and parameterized DefGate

## 0.13.0

### Breaking Changes

- The QVM API module has been expanded to include more types of requests. The existing methods in that module are now in the root of the QVM module and `NonZeroU16` is used for shot parameters throughout the library.

## 0.13.0-rc.0

### Breaking Changes

- The QVM API module has been expanded to include more types of requests. The existing methods in that module are now in the root of the QVM module and `NonZeroU16` is used for shot parameters throughout the library.

## 0.12.0

### Breaking Changes

- Loading a Qcs Client is now infallible and is the primary client used throughout the library. (#302)

## 0.12.0-rc.0

### Breaking Changes

- Loading a Qcs Client is now infallible and is the primary client used throughout the library. (#302)

## 0.11.0

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Features

- the default QCS Client configuration now respects environment variable overrides (#297)

### Fixes

- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.11.0-rc.4

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Features

- the default QCS Client configuration now respects environment variable overrides (#297)

### Fixes

- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.11.0-rc.3

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Features

- the default QCS Client configuration now respects environment variable overrides (#297)

### Fixes

- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.11.0-rc.2

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Fixes

- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.11.0-rc.1

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Fixes

- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.11.0-rc.0

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

## 0.10.0

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add methods for quil-c `conjugate_pauli_by_clifford` and `generate_randomized_benchmarking_sequence` (#280)
- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- fix broken tracing after merge conflicts (#278)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.23

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add methods for quil-c `conjugate_pauli_by_clifford` and `generate_randomized_benchmarking_sequence` (#280)
- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- fix broken tracing after merge conflicts (#278)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.22

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add methods for quil-c `conjugate_pauli_by_clifford` and `generate_randomized_benchmarking_sequence` (#280)
- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- fix broken tracing after merge conflicts (#278)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.21

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add methods for quil-c `conjugate_pauli_by_clifford` and `generate_randomized_benchmarking_sequence` (#280)
- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- fix broken tracing after merge conflicts (#278)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.20

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add methods for quil-c `conjugate_pauli_by_clifford` and `generate_randomized_benchmarking_sequence` (#280)
- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- fix broken tracing after merge conflicts (#278)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.19

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- fix broken tracing after merge conflicts (#278)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.18

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- add tracing support (#264)
- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.17

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.16

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- support endpoint_id job target (#262)
- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.15

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.14

### Breaking Changes

- support HTTPS_PROXY and HTTP_PROXY client proxies (#266)
- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.13

### Breaking Changes

- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.12

### Breaking Changes

- resolve breaking changes from api clients (#260)
- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.11

### Breaking Changes

- remove python 3.7 support (#259)
- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.10

### Breaking Changes

- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

### Fixes

- add missing types and fix others needed from pyquil (#257)

## 0.10.0-rc.9

### Breaking Changes

- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

## 0.10.0-rc.8

### Breaking Changes

- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

## 0.10.0-rc.7

### Breaking Changes

- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

## 0.10.0-rc.6

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

## 0.10.0-rc.5

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)

## 0.10.0-rc.4

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- export get_quilt_calibrations (#247)

## 0.10.0-rc.3

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

## 0.10.0-rc.2

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

## 0.10.0-rc.1

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

## 0.10.0-rc.0

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

## 0.9.2

### Fixes

- bump quil-rs to fix waveform issues (#228)

## 0.9.2-rc.0

### Fixes

- bump quil-rs to fix waveform issues (#228)

## 0.9.1

### Fixes

- Fetch Gateway address with pagination (#225)

## 0.9.1-rc.0

### Fixes

- Fetch Gateway address with pagination (#225)

## 0.9.0

### Breaking Changes

- compiler timeout is now configurable

## 0.9.0-rc.0

### Breaking Changes

- compiler timeout is now configurable

## 0.8.4

### Features

- add method for getting the version of the running quilc server (#206)

## 0.8.4-rc.0

### Features

- add method for getting the version of the running quilc server (#206)

## 0.8.3

### Fixes

- Add marker file to build types with bindings (#202)

## 0.8.3-rc.0

### Fixes

- Add marker file to build types with bindings (#202)

## 0.8.2

### Features

- support loading quilc and qvm server URLs from environment variables (#200)

## 0.8.2-rc.0

### Features

- support loading quilc and qvm server URLs from environment variables (#200)

## 0.8.1

### Fixes

- make grpc_api_url optional in settings.toml (#195)

## 0.8.1-rc.0

### Fixes

- make grpc_api_url optional in settings.toml (#195)

## 0.8.0

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- bump grpc API (#189)
- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.7

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- bump grpc API (#189)
- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.6

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- bump grpc API (#189)
- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.5

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.4

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.3

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.2

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.8.0-rc.1

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

### Fixes

- install protoc in github actions (#185)

## 0.8.0-rc.0

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

## 0.7.1-rc.22

### Features

- upgrade quil-rs dependency and fix issues (#172)
- python bindings (#145)

## 0.7.1-rc.21

### Features

- python bindings (#145)

## 0.7.1-rc.20

### Features

- python bindings (#145)

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
