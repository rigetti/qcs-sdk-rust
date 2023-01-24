use std::collections::HashMap;

use pyo3::{
    create_exception,
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    pyfunction,
    types::{PyComplex, PyDict, PyFloat, PyInt, PyList, PyString},
    Py, PyResult,
};
use qcs::{
    api::{
        ExecutionResult, ExecutionResults, Register, RewriteArithmeticResult, TranslationResult,
    },
    qpu::{
        quilc::{CompilerOpts, TargetDevice, DEFAULT_COMPILER_TIMEOUT},
        Qcs,
    },
};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_error, py_wrap_type, py_wrap_union_enum,
    wrap_error, ToPython,
};

create_init_submodule! {
    classes: [
        PyExecutionResult,
        PyExecutionResults,
        PyRegister,
        PyRewriteArithmeticResult,
        PyTranslationResult
    ],
    errors: [
        InvalidConfigError,
        ExecutionError,
        TranslationError,
        CompilationError,
        RewriteArithmeticError,
        DeviceIsaError,
        QcsSubmitError
    ],
    funcs: [
        compile,
        rewrite_arithmetic,
        translate,
        submit,
        retrieve_results,
        build_patch_values,
        get_quilc_version
    ],
}

py_wrap_data_struct! {
    PyRewriteArithmeticResult(RewriteArithmeticResult) as "RewriteArithmeticResult" {
        program: String => Py<PyString>,
        recalculation_table: Vec<String> => Py<PyList>
    }
}

py_wrap_data_struct! {
    PyTranslationResult(TranslationResult) as "TranslationResult" {
        program: String => Py<PyString>,
        ro_sources: Option<HashMap<String, String>> => Option<Py<PyDict>>
    }
}

py_wrap_type! {
    PyExecutionResults(ExecutionResults) as "ExecutionResults";
}

py_wrap_union_enum! {
    PyRegister(Register) as "Register" {
        f64: F64 => Vec<Py<PyFloat>>,
        i16: I16 => Vec<Py<PyInt>>,
        i32: I32 => Vec<Py<PyInt>>,
        i8: I8 => Vec<Py<PyInt>>,
        complex64: Complex64 => Vec<Py<PyComplex>>
    }
}

py_wrap_type! {
    PyExecutionResult(ExecutionResult) as "ExecutionResult";
}

create_exception!(qcs, InvalidConfigError, PyRuntimeError);
create_exception!(qcs, ExecutionError, PyRuntimeError);
create_exception!(qcs, TranslationError, PyRuntimeError);
create_exception!(qcs, CompilationError, PyRuntimeError);
create_exception!(qcs, RewriteArithmeticError, PyRuntimeError);
create_exception!(qcs, DeviceIsaError, PyValueError);

wrap_error!(SubmitError(qcs::api::SubmitError));
py_wrap_error!(api, SubmitError, QcsSubmitError, PyRuntimeError);

#[pyfunction(kwds = "**")]
pub fn compile<'a>(
    py: Python<'a>,
    quil: String,
    target_device: String,
    kwds: Option<&PyDict>,
) -> PyResult<&'a PyAny> {
    let target_device: TargetDevice =
        serde_json::from_str(&target_device).map_err(|e| DeviceIsaError::new_err(e.to_string()))?;

    let mut compiler_timeout = Some(DEFAULT_COMPILER_TIMEOUT);
    if let Some(kwargs) = kwds {
        if let Some(timeout_arg) = kwargs.get_item("timeout") {
            let timeout: Result<Option<u8>, _> = timeout_arg.extract();
            if let Ok(option) = timeout {
                compiler_timeout = option
            }
        }
    }

    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let options = CompilerOpts::default().with_timeout(compiler_timeout);
        let result = qcs::api::compile(&quil, target_device, &client, options)
            .map_err(|e| CompilationError::new_err(e.to_string()))?;
        Ok(result)
    })
}

#[pyfunction]
pub fn rewrite_arithmetic(native_quil: String) -> PyResult<PyRewriteArithmeticResult> {
    let native_program = native_quil
        .parse::<quil_rs::Program>()
        .map_err(|e| TranslationError::new_err(e.to_string()))?;
    let result = qcs::api::rewrite_arithmetic(native_program)
        .map_err(|e| RewriteArithmeticError::new_err(e.to_string()))?;
    let pyed = result.into();
    Ok(pyed)
}

#[pyfunction]
pub fn build_patch_values(
    py: Python<'_>,
    recalculation_table: Vec<String>,
    memory: HashMap<String, Vec<f64>>,
) -> PyResult<Py<PyDict>> {
    let memory = memory
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();
    let patch_values = qcs::api::build_patch_values(&recalculation_table, &memory)
        .map_err(TranslationError::new_err)?;
    patch_values
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<HashMap<_, _>>()
        .to_python(py)
}

#[pyfunction]
pub fn translate(
    py: Python<'_>,
    native_quil: String,
    num_shots: u16,
    quantum_processor_id: String,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let result = qcs::api::translate(&native_quil, num_shots, &quantum_processor_id, &client)
            .await
            .map_err(|e| TranslationError::new_err(e.to_string()))?;
        Python::with_gil(|py| PyTranslationResult::from(result).to_python(py))
    })
}

#[pyfunction]
pub fn submit(
    py: Python<'_>,
    program: String,
    patch_values: HashMap<String, Vec<f64>>,
    quantum_processor_id: String,
    use_gateway: bool,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?
            .with_use_gateway(use_gateway);
        let job_id = qcs::api::submit(&program, patch_values, &quantum_processor_id, &client)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))?;
        Ok(Python::with_gil(|_py| job_id))
    })
}

#[pyfunction]
pub fn retrieve_results(
    py: Python<'_>,
    job_id: String,
    quantum_processor_id: String,
    use_gateway: bool,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?
            .with_use_gateway(use_gateway);
        let results = qcs::api::retrieve_results(&job_id, &quantum_processor_id, &client)
            .await
            .map_err(|e| ExecutionError::new_err(e.to_string()))?;
        Ok(PyExecutionResults::from(results))
    })
}

#[pyfunction]
pub fn get_quilc_version(py: Python<'_>) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = Qcs::load()
            .await
            .map_err(|e| InvalidConfigError::new_err(e.to_string()))?;
        let version = qcs::api::get_quilc_version(&client)
            .map_err(|e| CompilationError::new_err(e.to_string()))?;
        Ok(version)
    })
}
