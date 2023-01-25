use std::{collections::HashMap, time::Duration};

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

#[pymethods]
impl PyReadoutMap {
    pub fn get_register_matrix(&self, register_name: String) -> Option<PyRegisterMatrix> {
        self.as_inner()
            .get_register_matrix(&register_name)
            .map(PyRegisterMatrix::from)
    }
}
