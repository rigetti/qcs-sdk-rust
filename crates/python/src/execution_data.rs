use std::time::Duration;

use numpy::{Complex64, PyArray2};
use pyo3::exceptions::PyKeyError;
use pyo3::{
    exceptions::PyValueError, pyclass, pymethods, types::PyDelta, Py, PyRef, PyRefMut, PyResult,
    Python,
};
use pyo3::{IntoPy, PyAny};
use qcs::{ExecutionData, RegisterMap, RegisterMatrix, ResultData};
use rigetti_pyo3::{
    py_wrap_data_struct, py_wrap_error, py_wrap_type, py_wrap_union_enum, wrap_error, PyTryFrom,
    PyWrapper, ToPython, ToPythonError,
};

use crate::qpu::PyQpuResultData;
use crate::qvm::PyQvmResultData;

py_wrap_union_enum! {
    PyResultData(ResultData) as "ResultData" {
        qpu: Qpu => PyQpuResultData,
        qvm: Qvm => PyQvmResultData
    }
}

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
}

py_wrap_data_struct! {
    PyExecutionData(ExecutionData) as "ExecutionData" {
        result_data: ResultData => PyResultData,
        duration: Option<Duration> => Option<Py<PyDelta>>
    }
}

#[pymethods]
impl PyExecutionData {
    #[new]
    fn __new__(py: Python<'_>, result_data: PyResultData, duration: Option<u64>) -> PyResult<Self> {
        Ok(Self(ExecutionData {
            result_data: ResultData::py_try_from(py, &result_data)?,
            duration: duration.map(Duration::from_micros),
        }))
    }
}

py_wrap_type! {
    #[pyo3(mapping)]
    PyRegisterMap(RegisterMap) as "RegisterMap";
}

py_wrap_type! {
    PyRegisterMatrix(RegisterMatrix) as "RegisterMatrix"
}

#[pymethods]
impl PyRegisterMatrix {
    fn to_ndarray(&self, py: Python<'_>) -> Py<PyAny> {
        self.as_integer(py)
            .map(|array| array.into_py(py))
            .or(self.as_real(py).map(|array| array.into_py(py)))
            .or(self.as_complex(py).map(|array| array.into_py(py)))
            .expect("A RegisterMatrix can't be any other type.")
    }

    #[staticmethod]
    fn from_integer(matrix: &PyArray2<i64>) -> PyRegisterMatrix {
        Self(RegisterMatrix::Integer(matrix.to_owned_array()))
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
    fn from_real(matrix: &PyArray2<f64>) -> PyRegisterMatrix {
        Self(RegisterMatrix::Real(matrix.to_owned_array()))
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
    fn from_complex(matrix: &PyArray2<Complex64>) -> PyRegisterMatrix {
        Self(RegisterMatrix::Complex(matrix.to_owned_array()))
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

    pub fn __contains__(&self, key: String) -> bool {
        self.as_inner().0.contains_key(&key)
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
