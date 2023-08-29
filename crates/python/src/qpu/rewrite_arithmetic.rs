//! Rewriting program arithmetic.
use std::{collections::HashMap, str::FromStr};

use pyo3::{exceptions::PyRuntimeError, pyclass, pyfunction, PyResult};
use quil_rs::{expression::Expression, quil::Quil};
use rigetti_pyo3::{create_init_submodule, py_wrap_error, ToPythonError};

create_init_submodule! {
    classes: [
        PyRewriteArithmeticResult
    ],
    errors: [
        BuildPatchValuesError,
        RewriteArithmeticError
    ],
    funcs: [
        build_patch_values,
        rewrite_arithmetic
    ],
}

/// Collection of errors that can result from rewriting arithmetic.
#[derive(Debug, thiserror::Error)]
pub enum RustRewriteArithmeticError {
    /// The Quil program could not be parsed.
    #[error("Could not parse program: {0}")]
    Program(#[from] quil_rs::program::ProgramError),

    /// Parameteric arithmetic in the Quil program could not be rewritten.
    #[error("Could not rewrite arithmetic: {0}")]
    Rewrite(#[from] qcs::qpu::rewrite_arithmetic::Error),
}

py_wrap_error!(
    rewrite_arithmetic,
    RustRewriteArithmeticError,
    RewriteArithmeticError,
    PyRuntimeError
);

/// The result of a call to [`rewrite_arithmetic()`] which provides the
/// information necessary to later patch-in memory values to a compiled program.
#[pyclass]
#[pyo3(name = "RewriteArithmeticResult")]
pub struct PyRewriteArithmeticResult {
    /// The rewritten program
    #[pyo3(get)]
    pub program: String,

    /// The expressions used to fill-in the `__SUBST` memory location. The
    /// expression index in this vec is the same as that in `__SUBST`.
    #[pyo3(get)]
    pub recalculation_table: Vec<String>,
}

/// Rewrite parametric arithmetic such that all gate parameters are only memory
/// references to newly declared memory location (`__SUBST`).
///
/// A "recalculation" table is provided which can be used to populate the memory
/// when needed (see `build_patch_values`).
///
/// # Errors
///
/// May return an error if the program fails to parse, or the parameter arithmetic
/// cannot be rewritten.
#[pyfunction]
pub fn rewrite_arithmetic(native_quil: String) -> PyResult<PyRewriteArithmeticResult> {
    let native_program = native_quil
        .parse::<quil_rs::Program>()
        .map_err(RustRewriteArithmeticError::from)
        .map_err(RustRewriteArithmeticError::to_py_err)?;

    let (program, index_set) = qcs::qpu::rewrite_arithmetic::rewrite_arithmetic(native_program)
        .map_err(RustRewriteArithmeticError::from)
        .map_err(RustRewriteArithmeticError::to_py_err)?;

    let program = program
        .to_quil()
        .expect("Successfully parsed program should convert to valid Quil.");
    let recalculation_table = index_set
        .into_iter()
        .map(|e| {
            e.to_quil().expect(
                "Expressions built from a successfully parsed program should convert to valid Quil.",
            )
        })
        .collect();

    Ok(PyRewriteArithmeticResult {
        program,
        recalculation_table,
    })
}

/// Collection of errors that can result from building patch values.
#[derive(Debug, thiserror::Error)]
pub enum RustBuildPatchValuesError {
    /// Failed to interpret the recalculation table.
    #[error("Unable to interpret recalculation table: {0:?}")]
    Substitutions(#[from] quil_rs::program::ParseProgramError<quil_rs::expression::Expression>),

    /// Failed to build patch values.
    #[error("Failed to build patch values: {0}")]
    PatchValues(String),
}

py_wrap_error!(
    rewrite_arithmetic,
    RustBuildPatchValuesError,
    BuildPatchValuesError,
    PyRuntimeError
);

/// Evaluate the expressions in `recalculation_table` using the numeric values
/// provided in `memory`.
///
/// # Errors
#[pyfunction]
pub fn build_patch_values(
    recalculation_table: Vec<String>,
    memory: HashMap<String, Vec<f64>>,
) -> PyResult<HashMap<String, Vec<f64>>> {
    let memory = memory
        .into_iter()
        .map(|(k, v)| (k.into_boxed_str(), v))
        .collect();
    let substitutions = recalculation_table
        .iter()
        .map(|expr| Expression::from_str(expr))
        .collect::<Result<_, _>>()
        .map_err(RustBuildPatchValuesError::from)
        .map_err(RustBuildPatchValuesError::to_py_err)?;
    let patch_values = qcs::qpu::rewrite_arithmetic::get_substitutions(&substitutions, &memory)
        .map_err(RustBuildPatchValuesError::PatchValues)
        .map_err(RustBuildPatchValuesError::to_py_err)?
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    Ok(patch_values)
}
