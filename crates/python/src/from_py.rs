use std::num::NonZeroU32;

use pyo3::{exceptions::PyValueError, PyAny, PyResult};

pub(crate) fn try_from_u32_to_non_zero_u32(int: u32) -> PyResult<NonZeroU32> {
    NonZeroU32::new(int).ok_or(PyValueError::new_err("value must be non-zero"))
}
pub(crate) fn non_zero_u32(obj: &PyAny) -> PyResult<NonZeroU32> {
    let value: u32 = obj.extract()?;
    try_from_u32_to_non_zero_u32(value)
}

pub(crate) fn optional_non_zero_u32(obj: &PyAny) -> PyResult<Option<NonZeroU32>> {
    let value: Option<u32> = obj.extract()?;
    match value {
        None => Ok(None),
        Some(int) => Ok(Some(try_from_u32_to_non_zero_u32(int)?)),
    }
}
