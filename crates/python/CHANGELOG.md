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
