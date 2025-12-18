use std::collections::HashMap;
use std::time::Duration;

use numpy::{Complex64, PyArray2};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    intern,
    prelude::*,
    types::{PyBytes, PyDelta, PyList},
    Bound, IntoPyObjectExt, Py, PyAny, PyRef, PyRefMut, PyResult, Python,
};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_complex_enum, gen_stub_pymethods};

use crate::qvm::QvmResultData;
use crate::{
    python::impl_repr,
    qpu::{result_data::MemoryValues, ReadoutValues},
    ExecutionData, RegisterMap, RegisterMatrix, ResultData,
};
use crate::{
    python::NonZeroU16,
    qpu::QpuResultData,
    qvm::http::{
        AddressRequest, ExpectationRequest, MultishotMeasureRequest, MultishotRequest,
        WavefunctionRequest,
    },
};

impl_repr!(ResultData);
impl_repr!(RawQpuReadoutData);

impl Default for ResultData {
    fn default() -> Self {
        ResultData::Qvm(QvmResultData::from_memory_map(HashMap::new()))
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl ResultData {
    /// Get the raw readout data from either QPU or QVM result.
    #[gen_stub(override_return_type(type_repr = "dict[str, list] | RawQPUReadoutData"))]
    fn to_raw_readout_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self {
            ResultData::Qpu(data) => data.to_raw_readout_data(py)?.into_bound_py_any(py),
            ResultData::Qvm(data) => data.to_raw_readout_data(py)?.into_bound_py_any(py),
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl ExecutionData {
    /// Python constructor for `ExecutionData`.
    ///
    /// `result_data` is optional here
    /// because pickling an object requires calling __new__ without arguments.
    #[new]
    #[pyo3(signature = (result_data=None, duration=None))]
    fn __new__(
        result_data: Option<ResultData>,
        duration: Option<Py<PyDelta>>,
        py: Python<'_>,
    ) -> PyResult<Self> {
        let result_data = result_data
            .unwrap_or_else(|| ResultData::Qvm(QvmResultData::from_memory_map(HashMap::new())));

        let duration = duration
            .map(|delta| {
                delta
                    .as_ref()
                    .call_method0(py, intern!(py, "total_seconds"))
                    .and_then(|result| result.extract::<f64>(py))
                    .map(Duration::from_secs_f64)
            })
            .transpose()?;

        Ok(Self {
            result_data,
            duration,
        })
    }

    fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        Ok(PyBytes::new(
            py,
            &serde_json::to_vec(self)
                .map_err(|e| PyRuntimeError::new_err(format!("failed to serialize: {e}")))?,
        ))
    }

    fn __setstate__<'py>(&mut self, _py: Python<'py>, state: Bound<'py, PyBytes>) -> PyResult<()> {
        let execution_data: ExecutionData = serde_json::from_slice(state.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("failed to deserialize: {e}")))?;
        *self = execution_data;
        Ok(())
    }
}

/// A 2 dimensional matrix of register values.
#[derive(Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass_complex_enum)]
#[pyclass(name = "RegisterMatrix", module = "qcs_sdk")]
pub(crate) enum PyRegisterMatrix {
    /// Integer register
    Integer(Py<PyArray2<i64>>),
    /// Real numbered register
    Real(Py<PyArray2<f64>>),
    /// Complex numbered register
    Complex(Py<PyArray2<Complex64>>),
}

impl<'a, 'py> FromPyObject<'a, 'py> for PyRegisterMatrix {
    type Error = PyErr;

    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(m) = ob.cast::<PyArray2<i64>>() {
            Ok(PyRegisterMatrix::Integer(m.to_owned().unbind()))
        } else if let Ok(m) = ob.cast::<PyArray2<f64>>() {
            Ok(PyRegisterMatrix::Real(m.to_owned().unbind()))
        } else if let Ok(m) = ob.cast::<PyArray2<Complex64>>() {
            Ok(PyRegisterMatrix::Complex(m.to_owned().unbind()))
        } else {
            Err(PyValueError::new_err(
                "expected a 2D numpy array of integers, reals, or complex numbers",
            ))
        }
    }
}

impl PyRegisterMatrix {
    fn from_register_matrix(py: Python<'_>, matrix: RegisterMatrix) -> Self {
        match matrix {
            RegisterMatrix::Integer(m) => {
                PyRegisterMatrix::Integer(PyArray2::from_owned_array(py, m).unbind())
            }
            RegisterMatrix::Real(m) => {
                PyRegisterMatrix::Real(PyArray2::from_owned_array(py, m).unbind())
            }
            RegisterMatrix::Complex(m) => {
                PyRegisterMatrix::Complex(PyArray2::from_owned_array(py, m).unbind())
            }
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl PyRegisterMatrix {
    fn to_ndarray(&self, py: Python<'_>) -> Py<PyAny> {
        match self {
            PyRegisterMatrix::Integer(m) => m.clone_ref(py).into_any(),
            PyRegisterMatrix::Real(m) => m.clone_ref(py).into_any(),
            PyRegisterMatrix::Complex(m) => m.clone_ref(py).into_any(),
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl RegisterMap {
    #[pyo3(name = "get_register_matrix")]
    fn py_get_register_matrix(
        &self,
        py: Python<'_>,
        register_name: &str,
    ) -> Option<PyRegisterMatrix> {
        self.0
            .get(register_name)
            .map(|matrix| PyRegisterMatrix::from_register_matrix(py, matrix.clone()))
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    fn __getitem__<'py>(&self, py: Python<'py>, item: &str) -> PyResult<PyRegisterMatrix> {
        self.py_get_register_matrix(py, item).ok_or_else(|| {
            pyo3::exceptions::PyKeyError::new_err(format!("Key {item} not found in RegisterMap"))
        })
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Py<RegisterMapKeysIter>> {
        Py::new(
            py,
            RegisterMapKeysIter {
                inner: self.0.clone().into_iter(),
            },
        )
    }

    fn keys(&self, py: Python<'_>) -> PyResult<Py<RegisterMapKeysIter>> {
        self.__iter__(py)
    }

    fn values(&self, py: Python<'_>) -> PyResult<Py<RegisterMapValuesIter>> {
        Py::new(
            py,
            RegisterMapValuesIter {
                inner: self.0.clone().into_iter(),
            },
        )
    }

    fn items(&self, py: Python<'_>) -> PyResult<Py<RegisterMapItemsIter>> {
        Py::new(
            py,
            RegisterMapItemsIter {
                inner: self.0.clone().into_iter(),
            },
        )
    }

    fn get(
        &self,
        key: &str,
        py: Python<'_>,
        default: Option<PyRegisterMatrix>,
    ) -> Option<PyRegisterMatrix> {
        self.__getitem__(py, key).ok().or(default)
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk")]
pub(crate) struct RegisterMapItemsIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl RegisterMapItemsIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(String, PyRegisterMatrix)> {
        slf.inner.next().map(|(register, matrix)| {
            (
                register,
                PyRegisterMatrix::from_register_matrix(slf.py(), matrix),
            )
        })
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk")]
pub(crate) struct RegisterMapKeysIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl RegisterMapKeysIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.inner.next().map(|(key, _)| key)
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk")]
pub(crate) struct RegisterMapValuesIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl RegisterMapValuesIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyRegisterMatrix> {
        slf.inner
            .next()
            .map(|(_, value)| PyRegisterMatrix::from_register_matrix(slf.py(), value))
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl QpuResultData {
    /// Construct a new `QPUResultData` from mappings and values.
    #[new]
    fn __new__(
        mappings: HashMap<String, String>,
        readout_values: HashMap<String, ReadoutValues>,
        memory_values: HashMap<String, MemoryValues>,
    ) -> Self {
        QpuResultData::from_mappings_and_values(mappings, readout_values, memory_values)
    }

    /// Get the raw readout data as a flattened structure.
    fn to_raw_readout_data(&self, py: Python<'_>) -> PyResult<RawQpuReadoutData> {
        Ok(RawQpuReadoutData {
            mappings: self.mappings().clone(),
            readout_values: self
                .readout_values()
                .iter()
                .map(|(register, values)| {
                    (match values {
                        ReadoutValues::Integer(values) => PyList::new(py, values),
                        ReadoutValues::Real(values) => PyList::new(py, values),
                        ReadoutValues::Complex(values) => PyList::new(py, values),
                    })
                    .map(|list| (register.clone(), list.unbind()))
                })
                .collect::<PyResult<_>>()?,
            memory_values: self
                .memory_values()
                .iter()
                .map(|(register, memory_values)| {
                    (match memory_values {
                        MemoryValues::Binary(values) => PyList::new(py, values),
                        MemoryValues::Integer(values) => PyList::new(py, values),
                        MemoryValues::Real(values) => PyList::new(py, values),
                    })
                    .map(|list| (register.clone(), list.unbind()))
                })
                .collect::<PyResult<_>>()?,
        })
    }
}

/// A wrapper type for data returned by the QPU in a more flat structure than
/// [`QpuResultData`] offers. This makes it more convenient to work with
/// the data if you don't care what type of number the readout values for
/// each register contains.
#[derive(Debug)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu", name = "RawQPUReadoutData", get_all)]
pub struct RawQpuReadoutData {
    pub mappings: HashMap<String, String>,
    pub readout_values: HashMap<String, Py<PyList>>,
    pub memory_values: HashMap<String, Py<PyList>>,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl QvmResultData {}
