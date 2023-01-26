use std::{collections::HashMap, time::Duration};

use numpy::{Complex64, PyArray2};
use pyo3::{
    exceptions::PyValueError,
    pymethods,
    types::{PyDelta, PyDict},
    Py, PyResult, Python,
};
use qcs::{ExecutionData, ReadoutData, ReadoutMap, RegisterMatrix};
use qcs_api_client_grpc::models::controller::{readout_values::Values, ReadoutValues};
use rigetti_pyo3::{
    py_wrap_data_struct, py_wrap_error, py_wrap_type, py_wrap_union_enum, wrap_error, PyWrapper,
    ToPython, ToPythonError,
};

use crate::register_data::PyRegisterData;
use crate::{grpc::models::controller::PyReadoutValuesValues, qpu::readout_data::PyQpuReadout};

py_wrap_union_enum! {
    PyReadoutData(ReadoutData) as "ReadoutData" {
        qpu: Qpu => PyQpuReadout,
        qvm: Qvm => HashMap<String, PyRegisterData> => Py<PyDict>
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
impl PyReadoutData {
    pub fn to_readout_map(&self, py: Python) -> PyResult<PyReadoutMap> {
        self.as_inner()
            .to_readout_map()
            .map_err(RegisterMatrixConversionError)
            .map_err(ToPythonError::to_py_err)?
            .to_python(py)
    }
}

py_wrap_data_struct! {
    PyExecutionData(ExecutionData) as "ExecutionData" {
        readout_data: ReadoutData => PyReadoutData,
        duration: Option<Duration> => Option<Py<PyDelta>>
    }
}

// From gRPC
py_wrap_data_struct! {
    PyReadoutValues(ReadoutValues) as "ReadoutValues" {
        values: Option<Values> => Option<PyReadoutValuesValues>
    }
}

py_wrap_type! {
    PyReadoutMap(ReadoutMap) as "ReadoutMap";
}

// TODO: Should be able to return inner matrix as ndarray
py_wrap_type! {
    PyRegisterMatrix(RegisterMatrix) as "RegisterMatrix";
}

impl PyRegisterMatrix {
    pub fn as_integer<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<i32>> {
        if let Some(matrix) = self.as_inner().as_integer() {
            Ok(PyArray2::from_array(py, matrix))
        } else {
            Err(PyValueError::new_err("not a integer register"))
        }
    }

    pub fn as_real<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<f64>> {
        if let Some(matrix) = self.as_inner().as_real() {
            Ok(PyArray2::from_array(py, matrix))
        } else {
            Err(PyValueError::new_err("not a real numbered register"))
        }
    }

    pub fn as_complex<'a>(&self, py: Python<'a>) -> PyResult<&'a PyArray2<Complex64>> {
        if let Some(matrix) = self.as_inner().as_complex() {
            Ok(PyArray2::from_array(py, matrix))
        } else {
            Err(PyValueError::new_err("not a complex numbered register"))
        }
    }
}

#[pymethods]
impl PyReadoutMap {
    pub fn get_register_matrix(&self, register_name: String) -> Option<PyRegisterMatrix> {
        self.as_inner()
            .get_register_matrix(&register_name)
            .map(PyRegisterMatrix::from)
    }
}
