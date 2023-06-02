use std::num::NonZeroU16;

use pyo3::{exceptions::PyValueError, PyAny, PyResult};

pub(crate) fn try_from_u16_to_non_zero_u16(int: u16) -> PyResult<NonZeroU16> {
    NonZeroU16::new(int).ok_or(PyValueError::new_err("value must be non-zero"))
}
pub(crate) fn non_zero_u16(obj: &PyAny) -> PyResult<NonZeroU16> {
    let value: u16 = obj.extract()?;
    try_from_u16_to_non_zero_u16(value)
}

pub(crate) fn optional_non_zero_u16(obj: &PyAny) -> PyResult<Option<NonZeroU16>> {
    let value: Option<u16> = obj.extract()?;
    match value {
        None => Ok(None),
        Some(int) => Ok(Some(try_from_u16_to_non_zero_u16(int)?)),
    }
}
