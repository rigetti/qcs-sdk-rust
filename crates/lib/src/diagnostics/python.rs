//! Provides diagnostics for the QCS SDK.

use pyo3::{ffi::c_str, prelude::*, types::IntoPyDict, IntoPyObjectExt};
use rigetti_pyo3::{create_init_submodule, sync::Awaitable};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::gen_stub_pyfunction;

use crate::diagnostics;

create_init_submodule! {
    funcs: [ get_report, get_report_async ],
}

/// Return a string describing the package and its environment for use in bug reporting and diagnosis.
///
/// Note: this format is not stable and its content may change between versions.
///
/// # Python Usage
///
/// ```python
/// import asyncio
/// from qcs_sdk import diagnostics
///
/// async def main():
///     return await diagnostics.get_report_async()
///
/// report = asyncio.run(main())
/// print(report)
/// ```
#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.diagnostics"))]
#[pyfunction]
pub(crate) fn get_report_async(py: Python<'_>) -> PyResult<Awaitable<'_, String>> {
    let py_part = get_py_report(py)?;

    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let rust_part = diagnostics::get_report().await;
        Ok(format!("{py_part}\n{rust_part}"))
    })
    .map(Into::into)
}

/// Return a string describing the package and its environment for use in bug reporting and diagnosis.
///
/// This is a synchronous wrapper around `get_report_async`.
/// Use that version in async environments.
///
/// Note: this format is not stable and its content may change between versions.
#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.diagnostics"))]
#[pyfunction]
pub(crate) fn get_report(py: Python<'_>) -> PyResult<String> {
    let py_part = get_py_report(py)?;
    let rust_part = pyo3_async_runtimes::tokio::run(py, async {
        PyResult::Ok(diagnostics::get_report().await)
    })?;
    Ok(format!("{py_part}\n{rust_part}"))
}

/// Get the details of the Python runtime environment.
fn get_py_report(py: Python<'_>) -> PyResult<String> {
    let version = env!("CARGO_PKG_VERSION");

    let locals = [
        ("sys", py.import("sys")?.into_bound_py_any(py)?),
        ("version", version.into_bound_py_any(py)?),
    ]
    .into_py_dict(py)?;

    let code = c_str!(
        r#"f'''qcs-sdk-python version: {version}
Python version: {sys.version}
Python implementation: {sys.implementation.name}
Python implementation version: {sys.implementation.version.major}.{sys.implementation.version.minor}.{sys.implementation.version.micro}
Python C API version: {sys.api_version}
Python executable: {sys.executable}
venv prefix: {sys.prefix}
platform: {sys.platform}'''"#
    );

    py.eval(code, None, Some(&locals))?.extract::<String>()
}
