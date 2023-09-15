use std::time::Duration;

use numpy::{Complex64, PyArray2};
use pyo3::exceptions::PyKeyError;
use pyo3::{
    exceptions::PyValueError, pyclass, pymethods, types::PyDelta, IntoPy, Py, PyObject, PyRef,
    PyRefMut, PyResult, Python, ToPyObject,
};
use qcs::{ExecutionData, RegisterMap, RegisterMatrix, ResultData};
use rigetti_pyo3::{
    impl_repr, py_wrap_data_struct, py_wrap_error, py_wrap_type, py_wrap_union_enum, wrap_error,
    PyTryFrom, PyWrapper, ToPython, ToPythonError,
};

use crate::qpu::PyQpuResultData;
use crate::qvm::PyQvmResultData;

py_wrap_union_enum! {
    PyResultData(ResultData) as "ResultData" {
        qpu: Qpu => PyQpuResultData,
        qvm: Qvm => PyQvmResultData
    }
}
impl_repr!(PyQpuResultData);

wrap_error!(RustRegisterMatrixConversionError(
    qcs::RegisterMatrixConversionError
));
py_wrap_error!(
    execution_data,
    RustRegisterMatrixConversionError,
    RegisterMatrixConversionError,
    PyValueError
);

#[pymethods]
impl PyResultData {
    pub fn to_register_map(&self, py: Python) -> PyResult<PyRegisterMap> {
        self.as_inner()
            .to_register_map()
            .map_err(RustRegisterMatrixConversionError)
            .map_err(ToPythonError::to_py_err)?
            .to_python(py)
    }

    pub fn to_raw_readout_data(&self, py: Python) -> PyResult<PyObject> {
        match self.as_inner() {
            ResultData::Qpu(_) => self
                .to_qpu(py)
                .map(|data| data.to_raw_readout_data(py).into_py(py)),
            ResultData::Qvm(_) => self
                .to_qvm(py)
                .map(|data| data.to_raw_readout_data(py).into_py(py)),
        }
    }
}

py_wrap_data_struct! {
    PyExecutionData(ExecutionData) as "ExecutionData" {
        result_data: ResultData => PyResultData,
        duration: Option<Duration> => Option<Py<PyDelta>>
    }
}
impl_repr!(PyExecutionData);

#[pymethods]
impl PyExecutionData {
    #[new]
    fn __new__(
        py: Python<'_>,
        result_data: PyResultData,
        duration: Option<Py<PyDelta>>,
    ) -> PyResult<Self> {
        Ok(Self(ExecutionData {
            result_data: ResultData::py_try_from(py, &result_data)?,
            duration: duration
                .map(|delta| {
                    delta
                        .as_ref(py)
                        .call_method0("total_seconds")
                        .map(|result| result.extract::<f64>())?
                        .map(Duration::from_secs_f64)
                })
                .transpose()?,
        }))
    }
}

py_wrap_type! {
    #[pyo3(mapping)]
    PyRegisterMap(RegisterMap) as "RegisterMap";
}
impl_repr!(PyRegisterMap);

py_wrap_type! {
    PyRegisterMatrix(RegisterMatrix) as "RegisterMatrix"
}
impl_repr!(PyRegisterMatrix);

#[pymethods]
impl PyRegisterMatrix {
    fn to_ndarray(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self.as_inner() {
            RegisterMatrix::Integer(_) => self.to_integer(py).map(|matrix| matrix.to_object(py)),
            RegisterMatrix::Real(_) => self.to_real(py).map(|matrix| matrix.to_object(py)),
            RegisterMatrix::Complex(_) => self.to_complex(py).map(|matrix| matrix.to_object(py)),
        }
    }

    #[staticmethod]
    fn from_integer(inner: &PyArray2<i64>) -> PyRegisterMatrix {
        Self(RegisterMatrix::Integer(inner.to_owned_array()))
    }

    fn to_integer<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<i64>> {
        if let Some(matrix) = self.as_inner().as_integer() {
            Ok(PyArray2::from_array(py, matrix))
        } else {
            Err(PyValueError::new_err("not a integer register"))
        }
    }

    fn as_integer<'a>(&self, py: Python<'a>) -> Option<&'a PyArray2<i64>> {
        if let Some(matrix) = self.as_inner().as_integer() {
            Some(PyArray2::from_array(py, matrix))
        } else {
            None
        }
    }

    fn is_integer(&self) -> bool {
        matches!(self.as_inner(), RegisterMatrix::Integer(_))
    }

    #[staticmethod]
    fn from_real(inner: &PyArray2<f64>) -> PyRegisterMatrix {
        Self(RegisterMatrix::Real(inner.to_owned_array()))
    }

    fn to_real<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<f64>> {
        if let Some(matrix) = self.as_inner().as_real() {
            Ok(PyArray2::from_array(py, matrix))
        } else {
            Err(PyValueError::new_err("not a real numbered register"))
        }
    }

    fn as_real<'a>(&self, py: Python<'a>) -> Option<&'a PyArray2<f64>> {
        if let Some(matrix) = self.as_inner().as_real() {
            Some(PyArray2::from_array(py, matrix))
        } else {
            None
        }
    }

    fn is_real(&self) -> bool {
        matches!(self.as_inner(), RegisterMatrix::Real(_))
    }

    #[staticmethod]
    fn from_complex(inner: &PyArray2<Complex64>) -> PyRegisterMatrix {
        Self(RegisterMatrix::Complex(inner.to_owned_array()))
    }

    fn to_complex<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<Complex64>> {
        if let Some(matrix) = self.as_inner().as_complex() {
            Ok(PyArray2::from_array(py, matrix))
        } else {
            Err(PyValueError::new_err("not a complex numbered register"))
        }
    }

    fn as_complex<'a>(&self, py: Python<'a>) -> Option<&'a PyArray2<Complex64>> {
        if let Some(matrix) = self.as_inner().as_complex() {
            Some(PyArray2::from_array(py, matrix))
        } else {
            None
        }
    }

    fn is_complex(&self) -> bool {
        matches!(self.as_inner(), RegisterMatrix::Complex(_))
    }
}

#[pymethods]
impl PyRegisterMap {
    pub fn get_register_matrix(&self, register_name: &str) -> Option<PyRegisterMatrix> {
        self.as_inner()
            .get_register_matrix(register_name)
            .map(PyRegisterMatrix::from)
    }

    pub fn __len__(&self) -> usize {
        self.as_inner().0.len()
    }

    pub fn __contains__(&self, key: &str) -> bool {
        self.as_inner().0.contains_key(key)
    }

    pub fn __getitem__(&self, item: &str) -> PyResult<PyRegisterMatrix> {
        self.get_register_matrix(item)
            .ok_or(PyKeyError::new_err(format!(
                "Key {item} not found in RegisterMap"
            )))
    }

    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyRegisterMapKeysIter>> {
        Py::new(
            py,
            PyRegisterMapKeysIter {
                inner: self.as_inner().0.clone().into_iter(),
            },
        )
    }

    pub fn keys(&self, py: Python<'_>) -> PyResult<Py<PyRegisterMapKeysIter>> {
        self.__iter__(py)
    }

    pub fn values(&self, py: Python<'_>) -> PyResult<Py<PyRegisterMapValuesIter>> {
        Py::new(
            py,
            PyRegisterMapValuesIter {
                inner: self.as_inner().0.clone().into_iter(),
            },
        )
    }

    pub fn items(&self, py: Python<'_>) -> PyResult<Py<PyRegisterMapItemsIter>> {
        Py::new(
            py,
            PyRegisterMapItemsIter {
                inner: self.as_inner().0.clone().into_iter(),
            },
        )
    }

    pub fn get(&self, key: &str, default: Option<PyRegisterMatrix>) -> Option<PyRegisterMatrix> {
        self.__getitem__(key).ok().or(default)
    }
}

#[pyclass]
pub struct PyRegisterMapItemsIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[pymethods]
impl PyRegisterMapItemsIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<(String, PyRegisterMatrix)> {
        slf.inner
            .next()
            .map(|(register, matrix)| (register, PyRegisterMatrix(matrix)))
    }
}

// The keys and values iterators are built on the iterator of the full
// `HashMap`, because the iterators returned by `keys()` and `values()`
// return an iterator with a _reference_ to the underlying `HashMap`.
// The reference would require these structs to specify a lifetime,
// which is incompatible with `#[pyclass]`.
#[pyclass]
pub struct PyRegisterMapKeysIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[pymethods]
impl PyRegisterMapKeysIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        slf.inner.next().map(|(register, _)| register)
    }
}

#[pyclass]
pub struct PyRegisterMapValuesIter {
    inner: std::collections::hash_map::IntoIter<String, RegisterMatrix>,
}

#[pymethods]
impl PyRegisterMapValuesIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyRegisterMatrix> {
        slf.inner.next().map(|(_, matrix)| PyRegisterMatrix(matrix))
    }
}
