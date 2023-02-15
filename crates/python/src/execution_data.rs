use std::time::Duration;

use numpy::{Complex64, PyArray2};
use pyo3::{exceptions::PyValueError, pymethods, types::PyDelta, Py, PyResult, Python};
use qcs::{ExecutionData, RegisterMap, RegisterMatrix, ResultData};
use qcs_api_client_grpc::models::controller::{readout_values::Values, ReadoutValues};
use rigetti_pyo3::{
    py_wrap_data_struct, py_wrap_error, py_wrap_type, py_wrap_union_enum, wrap_error, PyTryFrom,
    PyWrapper, ToPython, ToPythonError,
};

use crate::qvm::PyQvmResultData;
use crate::{grpc::models::controller::PyReadoutValuesValues, qpu::PyQpuResultData};

py_wrap_union_enum! {
    PyResultData(ResultData) as "ResultData" {
        qpu: Qpu => PyQpuResultData,
        qvm: Qvm => PyQvmResultData
    }
}

wrap_error!(RegisterMatrixConversionError(
    qcs::RegisterMatrixConversionError
));
py_wrap_error!(
    execution_data,
    RegisterMatrixConversionError,
    PyRegisterMatrixConversionError,
    PyValueError
);

#[pymethods]
impl PyResultData {
    fn to_register_map(&self, py: Python) -> PyResult<PyRegisterMap> {
        self.as_inner()
            .to_register_map()
            .map_err(RegisterMatrixConversionError)
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

// From gRPC
py_wrap_data_struct! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        values: Option<Values> => Option<PyReadoutValuesValues>
    }
}

py_wrap_type! {
    PyRegisterMap(RegisterMap) as "RegisterMap";
}

py_wrap_type! {
    PyRegisterMatrix(RegisterMatrix) as "RegisterMatrix"
}

#[pymethods]
impl PyRegisterMatrix {
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
    pub fn get_register_matrix(&self, register_name: String) -> Option<PyRegisterMatrix> {
        self.as_inner()
            .get_register_matrix(&register_name)
            .map(PyRegisterMatrix::from)
    }
}
