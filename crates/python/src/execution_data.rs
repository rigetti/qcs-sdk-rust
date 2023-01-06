use std::{collections::HashMap, time::Duration};

use pyo3::{
    types::{PyDelta, PyDict, PyString},
    Py,
};
use qcs::{Qvm, RegisterData};
use rigetti_pyo3::py_wrap_data_struct;

use crate::register_data::PyRegisterData;

py_wrap_data_struct! {
    PyQvm(Qvm) as "QVM" {
        registers: HashMap<Box<str>, RegisterData> => HashMap<Box<str>, PyRegisterData> => Py<PyDict>,
        duration: Option<Duration> => Option<Py<PyDelta>>
    }
}
