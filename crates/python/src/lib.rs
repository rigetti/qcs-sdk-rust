use pyo3::prelude::*;
use rigetti_pyo3::create_init_submodule;

use executable::QcsExecutionError;

pub mod api;
pub mod executable;
pub mod execution_data;
pub mod grpc;
pub mod qpu;
pub mod qvm;
pub mod register_data;

create_init_submodule! {
    classes: [
        execution_data::PyExecutionData,
        execution_data::PyResultData,
        execution_data::PyRegisterMap,
        execution_data::PyRegisterMatrix,
        executable::PyExecutable,
        executable::PyParameter,
        executable::PyJobHandle,
        executable::PyService,
        register_data::PyRegisterData,
        qpu::client::PyQcsClient
    ],
    errors: [
        QcsExecutionError,
        execution_data::PyRegisterMatrixConversionError
    ],
    funcs: [
        api::compile,
        api::rewrite_arithmetic,
        api::translate,
        api::submit,
        api::retrieve_results,
        api::build_patch_values,
        api::get_quilc_version,
        api::py_list_quantum_processors,
        qpu::isa::py_get_instruction_set_architecture
    ],
    submodules: [
        "api": api::init_submodule,
        "qpu": qpu::init_submodule,
        "qvm": qvm::init_submodule
    ],
}

#[pymodule]
fn qcs_sdk(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    init_submodule("qcs_sdk", py, m)
}

pub(crate) mod utils {
    /// Spawn and block on a future using the pyo3 tokio runtime.
    /// Useful for returning a synchronous `PyResult`.
    ///
    ///
    /// When used like the following:
    /// ```rs
    /// async fn say_hello(name: String) -> String {
    ///     format!("hello {name}")
    /// }
    ///
    /// #[pyo3(name="say_hello")]
    /// pub fn py_say_hello(name: String) -> PyResult<String> {
    ///     py_sync!(say_hello(name))
    /// }
    /// ```
    ///
    /// Becomes the associated "synchronous" python call:
    /// ```py
    /// assert say_hello("Rigetti") == "hello Rigetti"
    /// ```
    macro_rules! py_sync {
        ($body: expr) => {{
            let runtime = pyo3_asyncio::tokio::get_runtime();
            let handle = runtime.spawn($body);
            runtime
                .block_on(handle)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?
        }};
    }

    pub(crate) use py_sync;
}
