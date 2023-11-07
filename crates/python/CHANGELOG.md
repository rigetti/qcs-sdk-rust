## 0.13.2

### Features

- Update quil-rs (#383)

## 0.13.2-rc.0

### Features

- Update quil-rs (#383)

## 0.13.1

### Fixes

- update quil-rs to pull fixes (#381)

## 0.13.1-rc.0

### Fixes

- update quil-rs to pull fixes (#381)

## 0.13.0

### Breaking Changes

- use crate-defined translation options (#379)

### Features

- make Executable.get_qcs_client public (#380)

## 0.13.0-rc.1

### Breaking Changes

- use crate-defined translation options (#379)

### Features

- make Executable.get_qcs_client public (#380)

## 0.13.0-rc.0

### Breaking Changes

- use crate-defined translation options (#379)

## 0.12.9

### Features

- add a type alias for 'generic' gRPC connections (#378)

## 0.12.9-rc.0

### Features

- add a type alias for 'generic' gRPC connections (#378)

## 0.12.8

### Features

- provide access to the translation backend option (#373)

## 0.12.8-rc.0

### Features

- provide access to the translation backend option (#373)

## 0.12.7

### Fixes

- update quil version (#377)

## 0.12.7-rc.0

### Fixes

- update quil version (#377)

## 0.12.6

### Fixes

- update quil version (#376)

## 0.12.6-rc.0

### Fixes

- update quil version (#376)

## 0.12.5

### Fixes

- update quil-rs to pull Program::get_qubits fix (#375)

## 0.12.5-rc.0

### Fixes

- update quil-rs to pull Program::get_qubits fix (#375)

## 0.12.4

### Features

- Update quil-rs (#365)

## 0.12.4-rc.1

### Features

- Update quil-rs (#365)

## 0.12.4-rc.0

### Features

- Update quil-rs (#365)

## 0.12.3

### Features

- release with new grpc-channel construction method (#364)

## 0.12.3-rc.0

### Features

- release with new grpc-channel construction method (#364)

## 0.12.2

### Features

- update quil-rs (#363)

## 0.12.2-rc.0

### Features

- update quil-rs (#363)

## 0.12.1

### Features

- Update quil-rs (#361)
- Build & publish wheels for Windows (#351)

## 0.12.1-rc.1

### Features

- Update quil-rs (#361)
- Build & publish wheels for Windows (#351)

## 0.12.1-rc.0

### Features

- Build & publish wheels for Windows (#351)

## 0.12.0

### Breaking Changes

- much of the API code now requires an explicit client to be provided by the caller.

### Features

- Quilc/QVM clients (#353)

## 0.12.0-rc.0

### Breaking Changes

- much of the API code now requires an explicit client to be provided by the caller.

### Features

- Quilc/QVM clients (#353)

## 0.11.0

### Breaking Changes

- Constructors, iterators, and other utility methods have been added to make working with `ResultData`, `RegisterMap`, and others easier. The `ExecutionData` constructor now takes a `timedelta` for duration instead of a float. (#342)

### Features

- Update quil-rs (#350)

## 0.11.0-rc.1

### Breaking Changes

- Constructors, iterators, and other utility methods have been added to make working with `ResultData`, `RegisterMap`, and others easier. The `ExecutionData` constructor now takes a `timedelta` for duration instead of a float. (#342)

### Features

- Update quil-rs (#350)

## 0.11.0-rc.0

### Breaking Changes

- Constructors, iterators, and other utility methods have been added to make working with `ResultData`, `RegisterMap`, and others easier. The `ExecutionData` constructor now takes a `timedelta` for duration instead of a float. (#342)

## 0.10.8

### Features

- Default endpoints and gateway accessors are cached between requests (#337)

## 0.10.8-rc.0

### Features

- Default endpoints and gateway accessors are cached between requests (#337)

## 0.10.7

### Fixes

- The compiler, qpu, and qvm submodules are now packages again

## 0.10.7-rc.0

### Fixes

- The compiler, qpu, and qvm submodules are now packages again

## 0.10.6

### Fixes

- Return an error when execution results indicate that the job failed to run. (#331)

## 0.10.6-rc.0

### Fixes

- Return an error when execution results indicate that the job failed to run. (#331)

## 0.10.5

### Features

- Update quil-rs to 0.20.0, quil-py to 0.4.0

## 0.10.5-rc.1

### Features

- Update quil-rs to 0.20.0, quil-py to 0.4.0

## 0.10.5-rc.0

### Features

- Update quil-rs to 0.20.0, quil-py to 0.4.0

## 0.10.4

### Features

- A new `diagnostics` module has been added. The `get_report` function will print a summary of key diagnostic data to aid in debugging. (#334)

## 0.10.4-rc.0

### Features

- A new `diagnostics` module has been added. The `get_report` function will print a summary of key diagnostic data to aid in debugging. (#334)

## 0.10.3

### Features

- Broaden ExecutionOptions timeout_seconds type to float, add getters for properties

## 0.10.3-rc.1

### Features

- Broaden ExecutionOptions timeout_seconds type to float, add getters for properties

## 0.10.3-rc.0

### Features

- Broaden ExecutionOptions timeout_seconds type to float, add getters for properties

## 0.10.2

### Fixes

- add missing typehints for ExecutionOptions and ExecutionOptionsBuilder

## 0.10.2-rc.1

### Fixes

- add missing typehints for ExecutionOptions and ExecutionOptionsBuilder

## 0.10.2-rc.0

### Fixes

- add missing typehints for ExecutionOptions and ExecutionOptionsBuilder

## 0.10.1

### Features

- Add timeout to `ExecutionOptions` builder (#323)

### Fixes

- The MissingQubitBenchmark error has been demoted to a warning. (#322)

## 0.10.1-rc.1

### Features

- Add timeout to `ExecutionOptions` builder (#323)

### Fixes

- The MissingQubitBenchmark error has been demoted to a warning. (#322)

## 0.10.1-rc.0

### Fixes

- The MissingQubitBenchmark error has been demoted to a warning. (#322)

## 0.10.0

### Breaking Changes

- `submit` and `retrieve_results` now take an `ExecutionOptions` parameter for configuring the request. The contained `ConnectionStrategy` option is now used instead of setting `use_gateway` on the `QCSClient`

## 0.10.0-rc.1

### Breaking Changes

- `submit` and `retrieve_results` now take an `ExecutionOptions` parameter for configuring the request. The contained `ConnectionStrategy` option is now used instead of setting `use_gateway` on the `QCSClient`

## 0.10.0-rc.0

### Breaking Changes

- `submit` and `retrieve_results` now take an `ExecutionOptions` parameter for configuring the request. The contained `ConnectionStrategy` option is now used instead of setting `use_gateway` on the `QCSClient`

## 0.9.1

### Fixes

- Increase gRPC message size limit (#318)

## 0.9.1-rc.0

### Fixes

- Increase gRPC message size limit (#318)

## 0.9.0

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- The QCS Client now fallbacks to default values when loading incomplete settings. The default `grpc_api_url` has been updated. (#314)
- Integrate with logging (#311)
- TranslationOptions builder-like struct with Python bindings (#308)

## 0.9.0-rc.3

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- The QCS Client now fallbacks to default values when loading incomplete settings. The default `grpc_api_url` has been updated. (#314)
- Integrate with logging (#311)
- TranslationOptions builder-like struct with Python bindings (#308)

## 0.9.0-rc.2

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- Integrate with logging (#311)
- TranslationOptions builder-like struct with Python bindings (#308)

## 0.9.0-rc.1

### Breaking Changes

- QVM timeout can now be configured via a new options parameter (#309)
- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- TranslationOptions builder-like struct with Python bindings (#308)

## 0.9.0-rc.0

### Breaking Changes

- change to signature of `qpu::translate` to be generic over translation options

---------

### Features

- TranslationOptions builder-like struct with Python bindings (#308)

## 0.8.1

### Fixes

- Upgrade quil-rs, resolving some issues with parsing function calls and parameterized DefGate

## 0.8.1-rc.0

### Fixes

- Upgrade quil-rs, resolving some issues with parsing function calls and parameterized DefGate

## 0.8.0

### Breaking Changes

- The QVM API module has been expanded to include more types of requests. The existing methods in that module are now in the root of the QVM module and `NonZeroU16` is used for shot parameters throughout the library.

## 0.8.0-rc.0

### Breaking Changes

- The QVM API module has been expanded to include more types of requests. The existing methods in that module are now in the root of the QVM module and `NonZeroU16` is used for shot parameters throughout the library.

## 0.7.1

### Fixes

- The translate function no longer hangs infinitely (#305)

## 0.7.1-rc.0

### Fixes

- The translate function no longer hangs infinitely (#305)

## 0.7.0

### Breaking Changes

- Loading a Qcs Client is now infallible and is the primary client used throughout the library. (#302)

## 0.7.0-rc.0

### Breaking Changes

- Loading a Qcs Client is now infallible and is the primary client used throughout the library. (#302)

## 0.6.0

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Features

- the default QCS Client configuration now respects environment variable overrides (#297)

### Fixes

- check signals before blocking thread (#299)
- qvm_url is now set correctly when intializing a QCSClient (#296)
- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.6.0-rc.4

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Features

- the default QCS Client configuration now respects environment variable overrides (#297)

### Fixes

- check signals before blocking thread (#299)
- qvm_url is now set correctly when intializing a QCSClient (#296)
- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.6.0-rc.3

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Features

- the default QCS Client configuration now respects environment variable overrides (#297)

### Fixes

- qvm_url is now set correctly when intializing a QCSClient (#296)
- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.6.0-rc.2

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Fixes

- qvm_url is now set correctly when intializing a QCSClient (#296)
- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.6.0-rc.1

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

### Fixes

- Deserialize final_rewiring_values from None as an empty Vec (#293)

## 0.6.0-rc.0

### Breaking Changes

- `compile_program` now returns `CompilationResult`, containing the native program and `NativeQuilMetadata`

## 0.5.0

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
- implement get_instruction_set_architecture (#240)

### Fixes

- fix broken tracing after merge conflicts (#278)
- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.23

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
- implement get_instruction_set_architecture (#240)

### Fixes

- fix broken tracing after merge conflicts (#278)
- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.22

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
- implement get_instruction_set_architecture (#240)

### Fixes

- fix broken tracing after merge conflicts (#278)
- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.21

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
- implement get_instruction_set_architecture (#240)

### Fixes

- fix broken tracing after merge conflicts (#278)
- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.20

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
- implement get_instruction_set_architecture (#240)

### Fixes

- fix broken tracing after merge conflicts (#278)
- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.19

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
- implement get_instruction_set_architecture (#240)

### Fixes

- fix broken tracing after merge conflicts (#278)
- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.18

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
- implement get_instruction_set_architecture (#240)

### Fixes

- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.17

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
- implement get_instruction_set_architecture (#240)

### Fixes

- missing endpoint_id param added to api type annotations (#274)
- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.16

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
- implement get_instruction_set_architecture (#240)

### Fixes

- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.15

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
- implement get_instruction_set_architecture (#240)

### Fixes

- use qcs-api-client version with rustls and not openssl (#267)
- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.14

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
- implement get_instruction_set_architecture (#240)

### Fixes

- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.13

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
- implement get_instruction_set_architecture (#240)

### Fixes

- cargo deny warning from hermit-abi conflicts (#265)
- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.12

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
- implement get_instruction_set_architecture (#240)

### Fixes

- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.11

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
- implement get_instruction_set_architecture (#240)

### Fixes

- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.10

### Breaking Changes

- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

### Fixes

- add missing types and fix others needed from pyquil (#257)

## 0.5.0-rc.9

### Breaking Changes

- remove intermediate `api` module (#255)
- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.8

### Breaking Changes

- convert all pyo3 async functions to sync, use python name casing (#252)
- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.7

### Breaking Changes

- The `execution_data` module now provides `ExecutionData` as a replacement for both `Qvm` and `Qpu` structs. It serves a common interface for interacting with both result shapes when possible. See the `ExecutionData` documentation for more details on how to use it. (#223)
- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.6

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- export qcs client url settings (#249)
- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.5

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- support protoquil flag in compilation (rebase fix) (#243)
- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.4

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- export get_quilt_calibrations (#247)
- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.3

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.2

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

### Features

- implement get_instruction_set_architecture (#240)

## 0.5.0-rc.1

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

## 0.5.0-rc.0

### Breaking Changes

- implement Python wrappers for the full Rust API (#230)

## 0.4.2

### Fixes

- bump quil-rs to fix waveform issues (#228)

## 0.4.2-rc.0

### Fixes

- bump quil-rs to fix waveform issues (#228)

## 0.4.1

### Fixes

- Fetch Gateway address with pagination (#225)

## 0.4.1-rc.0

### Fixes

- Fetch Gateway address with pagination (#225)

## 0.4.0

### Breaking Changes

- compiler timeout is now configurable

## 0.4.0-rc.0

### Breaking Changes

- compiler timeout is now configurable

## 0.3.4

### Features

- add method for getting the version of the running quilc server (#206)

## 0.3.4-rc.0

### Features

- add method for getting the version of the running quilc server (#206)

## 0.3.3

### Fixes

- Add marker file to build types with bindings (#202)

## 0.3.3-rc.0

### Fixes

- Add marker file to build types with bindings (#202)

## 0.3.2

### Features

- support loading quilc and qvm server URLs from environment variables (#200)

### Fixes

- remove awaitable annotation on async functions (#199)
- add `async` to asynchronous function signatures, remove Awaitable from others (#198)

## 0.3.2-rc.2

### Features

- support loading quilc and qvm server URLs from environment variables (#200)

### Fixes

- remove awaitable annotation on async functions (#199)
- add `async` to asynchronous function signatures, remove Awaitable from others (#198)

## 0.3.2-rc.1

### Fixes

- remove awaitable annotation on async functions (#199)
- add `async` to asynchronous function signatures, remove Awaitable from others (#198)

## 0.3.2-rc.0

### Fixes

- add `async` to asynchronous function signatures, remove Awaitable from others (#198)

## 0.3.1

### Fixes

- make grpc_api_url optional in settings.toml (#195)

## 0.3.1-rc.0

### Fixes

- make grpc_api_url optional in settings.toml (#195)

## 0.3.0

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- bump grpc API (#189)
- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.7

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- bump grpc API (#189)
- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.6

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- bump grpc API (#189)
- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.5

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.4

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- move protoc install action to later (#188)
- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.3

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- bump qcs-api-client-common version (#187)
- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.2

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- include github token when installing protoc (#186)
- install protoc in github actions (#185)

## 0.3.0-rc.1

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

### Fixes

- install protoc in github actions (#185)

## 0.3.0-rc.0

### Breaking Changes

- RPCQ support has been dropped for translation and execution. Compilation (via quilc) still uses RPCQ.

### Features

- support gRPC translation and execution (#171)
- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

## 0.2.1-rc.22

### Features

- upgrade quil-rs dependency and fix issues (#172)
- add type hints
- python bindings (#145)

## 0.2.1-rc.21

### Features

- add type hints
- python bindings (#145)

## 0.2.1-rc.20

### Features

- add type hints
- python bindings (#145)

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
