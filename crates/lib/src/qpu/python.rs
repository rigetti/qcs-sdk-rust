//! Python bindings for the QPU module.

use std::time::Duration;

use numpy::Complex64;
use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, py_function_sync_async};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyfunction, gen_stub_pymethods};

use crate::{
    client::Qcs,
    python::{errors, execution_data::RawQpuReadoutData},
    qpu::{self, result_data::MemoryValues, QpuResultData, ReadoutValues},
};

// #[pyo3(name = "qpu", module = "qcs_sdk", submodule)]
create_init_submodule! {
    classes: [ QpuResultData, RawQpuReadoutData ],
    complex_enums: [ ReadoutValues, MemoryValues ],
    errors: [ errors::ListQuantumProcessorsError ],
    funcs: [ py_list_quantum_processors, py_list_quantum_processors_async ],
    submodules: [
        "api": qpu::api::python::init_submodule,
        "experimental": qpu::experimental::python::init_submodule,
        "isa": qpu::isa::python::init_submodule,
        "translation": qpu::translation::python::init_submodule
    ],
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ReadoutValues {
    #[new]
    fn __new__(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(values) = values.extract::<Vec<i64>>() {
            Ok(Self::Integer(values))
        } else if let Ok(values) = values.extract::<Vec<f64>>() {
            Ok(Self::Real(values))
        } else if let Ok(values) = values.extract::<Vec<Complex64>>() {
            Ok(Self::Complex(values))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "expected a list of integers, reals, or complex numbers",
            ))
        }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl MemoryValues {
    #[new]
    fn __new__(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(values) = values.extract::<Vec<u8>>() {
            Ok(Self::Binary(values))
        } else if let Ok(values) = values.extract::<Vec<i64>>() {
            Ok(Self::Integer(values))
        } else if let Ok(values) = values.extract::<Vec<f64>>() {
            Ok(Self::Real(values))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "expected a list of integers, reals, or complex numbers",
            ))
        }
    }
}

py_function_sync_async! {
    /// Returns all available Quantum Processor (QPU) IDs.
    ///
    /// :param client: The ``Qcs`` client to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    /// :param timeout: Maximum duration to wait for API calls to complete, in seconds.
    ///
    /// :raises ListQuantumProcessorsError: If the request to list available QPU IDs failed.
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
