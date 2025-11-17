use std::collections::HashMap;
use std::time::Duration;

use numpy::ToPyArray;
use pyo3::{
    exceptions::PyRuntimeError,
    intern,
    prelude::*,
    types::{PyBytes, PyDelta, PyList},
    Bound, IntoPyObjectExt, Py, PyAny, PyRef, PyRefMut, PyResult, Python,
};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

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
    pub fn to_raw_readout_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
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
    /// result_data is optional here
    /// because pickling an object requires calling __new__ without arguments.
    #[new]
    #[pyo3(signature = (result_data=None, duration=None))]
    pub fn __new__<'py>(
        result_data: Option<ResultData>,
        duration: Option<Py<PyDelta>>,
        py: Python<'py>,
    ) -> PyResult<Self> {
        let result_data = result_data.unwrap_or_else(|| {
            ResultData::Qvm(QvmResultData::from_memory_map(HashMap::new()))
        });

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

    pub fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        Ok(PyBytes::new(
            py,
            &serde_json::to_vec(self)
                .map_err(|e| PyRuntimeError::new_err(format!("failed to serialize: {e}")))?,
        ))
    }

    pub fn __setstate__<'py>(
        &mut self,
        _py: Python<'py>,
        state: Bound<'py, PyBytes>,
    ) -> PyResult<()> {
        let execution_data: ExecutionData = serde_json::from_slice(state.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("failed to deserialize: {e}")))?;
        *self = execution_data;
        Ok(())
    }
}

/// An enum representing every possible register type as a 2 dimensional matrix.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "RegisterMatrix", module = "qcs_sdk", frozen)]
pub(crate) struct PyRegisterMatrix(RegisterMatrix);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
impl PyRegisterMatrix {
    fn to_ndarray<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        match &self.0 {
            RegisterMatrix::Integer(m) => m.to_pyarray(py).into_any(),
            RegisterMatrix::Real(m) => m.to_pyarray(py).into_any(),
            RegisterMatrix::Complex(m) => m.to_pyarray(py).into_any(),
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl RegisterMap {
    #[pyo3(name = "get_register_matrix")]
    pub fn py_get_register_matrix(&self, register_name: &str) -> Option<PyRegisterMatrix> {
        self.0.get(register_name).cloned().map(PyRegisterMatrix)
    }

    pub fn __len__(&self) -> usize {
        self.0.len()
    }

    pub fn __contains__(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn __getitem__(&self, item: &str) -> PyResult<PyRegisterMatrix> {
        self.py_get_register_matrix(item).ok_or_else(|| {
            pyo3::exceptions::PyKeyError::new_err(format!("Key {item} not found in RegisterMap"))
        })
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Py<RegisterMapKeysIter>> {
        Py::new(
            py,
            RegisterMapKeysIter {
                inner: self.0.clone().into_iter(),
            },
        )
    }

    pub fn keys(&self, py: Python<'_>) -> PyResult<Py<RegisterMapKeysIter>> {
        self.__iter__(py)
    }

    pub fn values(&self, py: Python<'_>) -> PyResult<Py<RegisterMapValuesIter>> {
        Py::new(
            py,
            RegisterMapValuesIter {
                inner: self.0.clone().into_iter(),
            },
        )
    }

    pub fn items(&self, py: Python<'_>) -> PyResult<Py<RegisterMapItemsIter>> {
        Py::new(
            py,
            RegisterMapItemsIter {
                inner: self.0.clone().into_iter(),
            },
        )
    }

    pub fn get(&self, key: &str, default: Option<PyRegisterMatrix>) -> Option<PyRegisterMatrix> {
        self.__getitem__(key).ok().or(default)
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk")]
pub struct RegisterMapItemsIter {
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
        slf.inner
            .next()
            .map(|(register, matrix)| (register, PyRegisterMatrix(matrix)))
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk")]
pub struct RegisterMapKeysIter {
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
pub struct RegisterMapValuesIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl RegisterMapValuesIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<PyRegisterMatrix> {
        self.inner.next().map(|(_, value)| PyRegisterMatrix(value))
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl QpuResultData {
    /// Construct a new `QPUResultData` from mappings and values.
    #[new]
    pub fn __new__(
        mappings: HashMap<String, String>,
        readout_values: HashMap<String, ReadoutValues>,
        memory_values: HashMap<String, MemoryValues>,
    ) -> Self {
        QpuResultData::from_mappings_and_values(mappings, readout_values, memory_values)
    }

    /// Get the raw readout data as a flattened structure.
    pub fn to_raw_readout_data<'py>(&self, py: Python<'py>) -> PyResult<RawQpuReadoutData> {
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

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl MultishotRequest {
    /// Creates a new `MultishotRequest` with the given parameters.
    #[new]
    #[pyo3(signature = (compiled_quil, trials, addresses, measurement_noise=None, gate_noise=None, rng_seed=None))]
    pub fn __new__(
        compiled_quil: String,
        trials: NonZeroU16,
        addresses: HashMap<String, AddressRequest>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> PyResult<Self> {
        Ok(MultishotRequest::new(
            compiled_quil,
            trials.0,
            addresses,
            measurement_noise,
            gate_noise,
            rng_seed,
        ))
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl MultishotMeasureRequest {
    /// Construct a new `MultishotMeasureRequest` using the given parameters.
    #[new]
    #[pyo3(signature = (compiled_quil, trials, qubits, measurement_noise=None, gate_noise=None, rng_seed=None))]
    pub fn __new__(
        compiled_quil: String,
        trials: NonZeroU16,
        qubits: Vec<u64>,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> PyResult<Self> {
        Ok(MultishotMeasureRequest::new(
            compiled_quil,
            trials.0,
            &qubits,
            measurement_noise,
            gate_noise,
            rng_seed,
        ))
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl ExpectationRequest {
    /// Creates a new `ExpectationRequest` using the given parameters.
    #[new]
    #[pyo3(signature = (state_preparation, operators, rng_seed=None))]
    pub fn __new__(
        state_preparation: String,
        operators: Vec<String>,
        rng_seed: Option<i64>,
    ) -> Self {
        ExpectationRequest::new(state_preparation, &operators, rng_seed)
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl WavefunctionRequest {
    /// Create a new `WavefunctionRequest` with the given parameters.
    #[new]
    #[pyo3(signature = (compiled_quil, measurement_noise=None, gate_noise=None, rng_seed=None))]
    pub fn __new__(
        compiled_quil: String,
        measurement_noise: Option<(f64, f64, f64)>,
        gate_noise: Option<(f64, f64, f64)>,
        rng_seed: Option<i64>,
    ) -> Self {
        WavefunctionRequest::new(compiled_quil, measurement_noise, gate_noise, rng_seed)
    }
}
