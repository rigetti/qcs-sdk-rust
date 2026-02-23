//! Python bindings for the QPU module.

use std::time::Duration;

use numpy::Complex64;
use pyo3::{
    prelude::*,
    types::{PyFloat, PyInt, PyList},
};
use rigetti_pyo3::{create_init_submodule, impl_repr, py_function_sync_async};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyfunction, gen_stub_pymethods};

use crate::{
    client::Qcs,
    python::{errors, execution_data::RawQpuReadoutData},
    qpu::{self, result_data::MemoryValues, QpuResultData, ReadoutValues},
};

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

impl_repr!(QpuResultData);
impl_repr!(RawQpuReadoutData);

#[derive(FromPyObject, IntoPyObject)]
enum PyReadoutValues {
    Integer(Vec<i64>),
    Real(Vec<f64>),
    Complex(Vec<Complex64>),
}

#[derive(FromPyObject, IntoPyObject)]
enum PyMemoryValues {
    // In Python, Binary and Integer are both list[int], so the extraction here is just Vec<i64>.
    Integer(Vec<i64>),
    Real(Vec<f64>),
}

#[cfg(feature = "stubs")]
pyo3_stub_gen::impl_stub_type!(PyReadoutValues = Vec<i64> | Vec<f64> | Vec<Complex64>);

#[cfg(feature = "stubs")]
pyo3_stub_gen::impl_stub_type!(PyMemoryValues = Vec<i64> | Vec<f64>);

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ReadoutValues {
    #[new]
    fn __new__(values: PyReadoutValues) -> Self {
        match values {
            PyReadoutValues::Integer(values) => Self::Integer(values),
            PyReadoutValues::Real(values) => Self::Real(values),
            PyReadoutValues::Complex(values) => Self::Complex(values),
        }
    }

    fn __getnewargs__(&self) -> (PyReadoutValues,) {
        (self.inner(),)
    }

    fn inner(&self) -> PyReadoutValues {
        match self {
            Self::Integer(values) => PyReadoutValues::Integer(values.clone()),
            Self::Real(values) => PyReadoutValues::Real(values.clone()),
            Self::Complex(values) => PyReadoutValues::Complex(values.clone()),
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl MemoryValues {
    #[new]
    fn __new__(values: PyMemoryValues) -> Self {
        match values {
            PyMemoryValues::Integer(values) => {
                if values.iter().all(|&v| v == 0 || v == 1) {
                    #[allow(
                        clippy::cast_sign_loss,
                        clippy::cast_possible_truncation,
                        reason = "we know the values are 0 or 1"
                    )]
                    Self::Binary(values.into_iter().map(|v| v as u8).collect())
                } else {
                    Self::Integer(values)
                }
            }
            PyMemoryValues::Real(values) => Self::Real(values),
        }
    }

    #[gen_stub(override_return_type(
        type_repr = "tuple[builtins.list[builtins.int] | builtins.list[builtins.float]]"
    ))]
    fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, PyList>,)> {
        Ok((self.inner(py)?,))
    }

    #[gen_stub(override_return_type(
        type_repr = "builtins.list[builtins.int] | builtins.list[builtins.float]"
    ))]
    fn inner<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        // This is implemented manually because otherwise we'd need to clone the list twice:
        // once to take ownership of the Rust values, and again to create the Python values.
        match self {
            Self::Binary(values) => PyList::new(py, values.iter().map(|v| PyInt::new(py, *v))),
            Self::Integer(values) => PyList::new(py, values.iter().map(|v| PyInt::new(py, *v))),
            Self::Real(values) => PyList::new(py, values.iter().map(|v| PyFloat::new(py, *v))),
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
