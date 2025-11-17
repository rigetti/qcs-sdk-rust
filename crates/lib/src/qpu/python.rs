use std::time::Duration;

use pyo3::{prelude::*, wrap_pymodule};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::gen_stub_pyfunction;

use crate::{
    client::Qcs,
    python::{errors, execution_data::RawQpuReadoutData, py_function_sync_async},
    qpu::{
        self, api, experimental, isa, result_data::MemoryValues, translation, QpuResultData,
        ReadoutValues,
    },
};

#[pymodule]
#[pyo3(name = "qpu", module = "qcs_sdk", submodule)]
pub(crate) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add(
        "ListQuantumProcessorsError",
        py.get_type::<errors::ListQuantumProcessorsError>(),
    )?;

    m.add_class::<QpuResultData>()?;
    m.add_class::<RawQpuReadoutData>()?;
    m.add_class::<ReadoutValues>()?;
    m.add_class::<MemoryValues>()?;

    m.add_function(wrap_pyfunction!(py_list_quantum_processors, m)?)?;
    m.add_function(wrap_pyfunction!(py_list_quantum_processors_async, m)?)?;

    m.add_wrapped(wrap_pymodule!(api::python::init_module))?;
    m.add_wrapped(wrap_pymodule!(experimental::python::init_module))?;
    m.add_wrapped(wrap_pymodule!(isa::python::init_module))?;
    m.add_wrapped(wrap_pymodule!(translation::python::init_module))?;
    api::python::init_module(m)?;
    experimental::python::init_module(m)?;
    isa::python::init_module(m)?;
    translation::python::init_module(m)?;

    Ok(())
}

py_function_sync_async! {
    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu"))]
    #[pyfunction]
    #[pyo3(signature = (client = None, timeout = None))]
    async fn list_quantum_processors(
        client: Option<Qcs>,
        timeout: Option<f64>,
    ) -> PyResult<Vec<String>> {
        let client = client.unwrap_or_else(Qcs::load);
        let timeout = timeout.map(Duration::from_secs_f64);
        qpu::list_quantum_processors(&client, timeout)
            .await
            .map_err(Into::into)
    }
}
