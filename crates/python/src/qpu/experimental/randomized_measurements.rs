use std::collections::HashMap;
use std::str::FromStr;

use numpy::Complex64;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use quil_rs::program::ProgramError;

use super::random::PyPrngSeedValue;
use qcs::qpu::experimental::randomized_measurements::{
    QubitRandomization, RandomizedMeasurements, UnitarySet,
};
use quil_rs::quil::Quil;
use rigetti_pyo3::pyo3::exceptions::PyValueError;
use rigetti_pyo3::{create_init_submodule, py_wrap_error, wrap_error, ToPythonError};

use quil_rs::instruction::{Declaration, Measurement, MemoryReference, Qubit, ScalarType, Vector};

create_init_submodule! {
    classes: [
        PyRandomizedMeasurements,
        PyUnitaryParameterDeclaration,
        PyRandomizedMeasurement,
        PyQubitRandomization
    ],
    errors: [
        RandomizedMeasurementsError,
        AppendToProgramError,
        ToParametersError
    ],
}

#[pyclass(name = "QubitRandomization")]
#[derive(Clone)]
struct PyQubitRandomization {
    inner: QubitRandomization,
}

#[pymethods]
impl PyQubitRandomization {
    #[getter]
    fn seed_declaration(&self) -> PyUnitaryParameterDeclaration {
        self.inner.get_seed_declaration().clone().into()
    }

    #[getter]
    fn destination_declaration(&self) -> PyUnitaryParameterDeclaration {
        self.inner.get_destination_declaration().clone().into()
    }

    #[getter]
    fn measurement(&self) -> PyResult<PyRandomizedMeasurement> {
        self.inner.get_measurement().clone().try_into()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum UnitarySetError {
    #[error("failed to call method `{method_name}` on unitary set: {error}")]
    CallMethod {
        method_name: &'static str,
        #[source]
        error: PyErr,
    },
    #[error("failed to extract return type `{expected_type}` for unitary set method `{method_name}`: {error}")]
    MethodReturnType {
        method_name: &'static str,
        expected_type: &'static str,
        #[source]
        error: PyErr,
    },
    #[error("failed to parse unitary set instructions: {0}")]
    InvalidInstructions(#[from] ProgramError),
}

impl ToPythonError for UnitarySetError {
    fn to_py_err(self) -> PyErr {
        PyRuntimeError::new_err(self.to_string())
    }
}

#[derive(Clone)]
pub(super) struct UnitarySetWrapper {
    inner: Py<PyAny>,
    unitary_count: usize,
    parameters_per_unitary: usize,
}

macro_rules! call_method {
    ($py:ident, $inner:expr, $method_name:expr, $expected_type:ty) => {
        call_method!($py, $inner, $method_name, (), $expected_type)
    };
    ($py:ident, $inner:expr, $method_name:expr, $args:expr, $expected_type:ty) => {
        $inner
            .call_method($py, $method_name, $args, None)
            .map_err(|e| UnitarySetError::CallMethod {
                method_name: $method_name,
                error: e,
            })
            .and_then(|result| {
                result.extract::<$expected_type>($py).map_err(|e| {
                    UnitarySetError::MethodReturnType {
                        method_name: $method_name,
                        expected_type: std::any::type_name::<$expected_type>(),
                        error: e,
                    }
                })
            })
    };
}

impl UnitarySetWrapper {
    pub(super) fn try_new(py: Python, inner: Py<PyAny>) -> PyResult<Self> {
        let unitary_count =
            call_method!(py, inner, "unitary_count", usize).map_err(ToPythonError::to_py_err)?;
        let parameters_per_unitary = call_method!(py, inner, "parameters_per_unitary", usize)
            .map_err(ToPythonError::to_py_err)?;
        Ok(Self {
            inner,
            unitary_count,
            parameters_per_unitary,
        })
    }
}

impl UnitarySet for UnitarySetWrapper {
    type Error = UnitarySetError;

    fn unitary_count(&self) -> usize {
        self.unitary_count
    }

    fn parameters_per_unitary(&self) -> usize {
        self.parameters_per_unitary
    }

    fn to_parameters(&self) -> Result<Vec<f64>, UnitarySetError> {
        Python::with_gil(|py| call_method!(py, self.inner, "to_parameters", Vec<f64>))
    }

    fn to_instructions(
        &self,
        qubit_randomizations: &[QubitRandomization],
    ) -> Result<Vec<quil_rs::instruction::Instruction>, UnitarySetError> {
        Python::with_gil(|py| {
            let qubit_randomizations = pyo3::types::PyList::new(
                py,
                qubit_randomizations
                    .iter()
                    .cloned()
                    .map(|inner| PyQubitRandomization { inner }.into_py(py))
                    .collect::<Vec<_>>(),
            );

            let quil = call_method!(
                py,
                self.inner,
                "to_instructions",
                (qubit_randomizations,),
                String
            )?;
            Ok(quil_rs::Program::from_str(quil.as_str())
                .map_err(UnitarySetError::InvalidInstructions)?
                .into_instructions())
        })
    }
}
#[pyclass(name = "RandomizedMeasurement")]
#[derive(Clone)]
pub(super) struct PyRandomizedMeasurement {
    qubit: u64,
    memory_reference_name: String,
    memory_reference_index: u64,
}

#[pymethods]
impl PyRandomizedMeasurement {
    #[new]
    fn new(qubit: u64, memory_reference_name: String, memory_reference_index: u64) -> Self {
        Self {
            qubit,
            memory_reference_name,
            memory_reference_index,
        }
    }

    #[getter]
    fn get_qubit(&self) -> u64 {
        self.qubit
    }

    #[getter]
    fn get_memory_reference_name(&self) -> String {
        self.memory_reference_name.clone()
    }

    #[getter]
    fn get_memory_reference_index(&self) -> u64 {
        self.memory_reference_index
    }
}

impl From<PyRandomizedMeasurement> for Measurement {
    fn from(py_measurement: PyRandomizedMeasurement) -> Self {
        Self {
            qubit: Qubit::Fixed(py_measurement.qubit),
            target: Some(MemoryReference {
                name: py_measurement.memory_reference_name.clone(),
                index: py_measurement.memory_reference_index,
            }),
        }
    }
}

impl TryFrom<Measurement> for PyRandomizedMeasurement {
    type Error = PyErr;

    fn try_from(measurement: Measurement) -> PyResult<Self> {
        let qubit = match measurement.qubit {
            Qubit::Fixed(qubit) => qubit,
            _ => return Err(PyValueError::new_err("only fixed qubits are supported").to_py_err()),
        };
        let (memory_reference_name, memory_reference_index) = measurement
            .target
            .map(|target| (target.name.clone(), target.index))
            .ok_or(PyValueError::new_err("measurement target must be set").to_py_err())?;
        Ok(Self {
            qubit,
            memory_reference_name,
            memory_reference_index,
        })
    }
}

#[pyclass(name = "UnitaryParameterDeclaration")]
#[derive(Clone)]
pub(super) struct PyUnitaryParameterDeclaration {
    inner: Declaration,
}

#[pymethods]
impl PyUnitaryParameterDeclaration {
    #[new]
    fn new(name: String, length: u64) -> Self {
        Self {
            inner: Declaration {
                name,
                size: Vector::new(ScalarType::Real, length),
                sharing: None,
            },
        }
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    #[getter]
    fn length(&self) -> u64 {
        self.inner.size.length
    }
}

impl From<PyUnitaryParameterDeclaration> for Declaration {
    fn from(py_measurement: PyUnitaryParameterDeclaration) -> Self {
        py_measurement.inner
    }
}

impl From<Declaration> for PyUnitaryParameterDeclaration {
    fn from(declaration: Declaration) -> Self {
        Self { inner: declaration }
    }
}

wrap_error!(RustRandomizedMeasurementsError(
    qcs::qpu::experimental::randomized_measurements::Error
));
py_wrap_error!(
    experimental,
    RustRandomizedMeasurementsError,
    RandomizedMeasurementsError,
    PyValueError
);

type AppendToProgramUnitarySetError =
    qcs::qpu::experimental::randomized_measurements::AppendToProgramError<UnitarySetError>;

wrap_error!(RustAppendToProgramError(AppendToProgramUnitarySetError));
py_wrap_error!(
    experimental,
    RustAppendToProgramError,
    AppendToProgramError,
    PyValueError
);

type ToParametersUnitarySetError =
    qcs::qpu::experimental::randomized_measurements::ToParametersError<UnitarySetError>;

wrap_error!(RustToParametersError(ToParametersUnitarySetError));
py_wrap_error!(
    experimental,
    RustToParametersError,
    ToParametersError,
    PyValueError
);

#[pyclass(name = "RandomizedMeasurements")]
#[derive(Clone)]
struct PyRandomizedMeasurements {
    inner: RandomizedMeasurements<UnitarySetWrapper>,
}

#[pymethods]
impl PyRandomizedMeasurements {
    #[new]
    #[pyo3(signature = (measurements, unitary_set, leading_delay = 1e-5))]
    fn new(
        py: Python,
        measurements: Vec<PyRandomizedMeasurement>,
        unitary_set: Py<PyAny>,
        leading_delay: f64,
    ) -> PyResult<Self> {
        let unitary_set = UnitarySetWrapper::try_new(py, unitary_set)?;
        RandomizedMeasurements::try_new(
            measurements.into_iter().map(Measurement::from).collect(),
            unitary_set,
            quil_rs::expression::Expression::Number(Complex64 {
                re: leading_delay,
                im: 0.0,
            }),
        )
        .map(|inner| Self { inner })
        .map_err(RustRandomizedMeasurementsError::from)
        .map_err(RustRandomizedMeasurementsError::to_py_err)
    }

    fn append_to_program(&self, target_program: String) -> PyResult<String> {
        let target_program = quil_rs::Program::from_str(target_program.as_str())
            .map_err(|e| PyValueError::new_err(format!("failed to parse target program: {e}")))?;
        self.inner
            .append_to_program(target_program)
            .map_err(RustAppendToProgramError::from)
            .map_err(RustAppendToProgramError::to_py_err)
            .and_then(|program| {
                program.to_quil().map_err(|e| {
                    PyRuntimeError::new_err(format!("failed to write program as Quil: {e}"))
                })
            })
    }

    fn to_parameters(
        &self,
        seed_values: HashMap<u64, PyPrngSeedValue>,
    ) -> PyResult<HashMap<String, Vec<f64>>> {
        self.inner
            .to_parameters(&seed_values.into_iter().map(|(k, v)| (k, v.inner)).collect())
            .map(|parameters| {
                parameters
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect()
            })
            .map_err(RustToParametersError::from)
            .map_err(RustToParametersError::to_py_err)
    }

    fn get_random_indices(
        &self,
        seed_values: HashMap<u64, PyPrngSeedValue>,
        shot_count: u32,
    ) -> HashMap<u64, Vec<u8>> {
        self.inner.get_random_indices(
            &seed_values.into_iter().map(|(k, v)| (k, v.inner)).collect(),
            shot_count,
        )
    }
}
