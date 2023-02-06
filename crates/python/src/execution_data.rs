use std::{collections::HashMap, time::Duration};

use pyo3::{
    pymethods,
    types::{PyDelta, PyDict},
    Py, PyResult, Python,
};
use qcs::{Qpu, Qvm, ReadoutMap, RegisterData};
use qcs_api_client_grpc::models::controller::{readout_values::Values, ReadoutValues};
use rigetti_pyo3::{py_wrap_data_struct, py_wrap_type, PyWrapper, ToPython};

use crate::grpc::models::controller::PyReadoutValuesValues;
use crate::register_data::PyRegisterData;

py_wrap_data_struct! {
    PyQvm(Qvm) as "QVM" {
        registers: HashMap<String, RegisterData> => HashMap<String, PyRegisterData> => Py<PyDict>,
        duration: Option<Duration> => Option<Py<PyDelta>>
    }
}

py_wrap_data_struct! {
    PyQpu(Qpu) as "QPU" {
        readout_data: ReadoutMap => PyReadoutMap,
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

#[pymethods]
impl PyReadoutMap {
    pub fn get_readout_values(&self, field: String, index: u64) -> Option<PyReadoutValues> {
        self.as_inner()
            .get_readout_values(field, index)
            .map(PyReadoutValues::from)
    }

    pub fn get_readout_values_for_field(
        &self,
        py: Python,
        field: &str,
    ) -> PyResult<Option<Vec<Option<PyReadoutValues>>>> {
        let op = self.as_inner().get_readout_values_for_field(field)?;
        op.map(|list| {
            list.into_iter()
                .map(|op| op.to_python(py))
                .collect::<PyResult<_>>()
        })
        .transpose()
    }
}
