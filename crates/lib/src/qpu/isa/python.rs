use pyo3::prelude::*;

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use qcs_api_client_openapi::models::{
    Architecture, Characteristic, Edge, Family, InstructionSetArchitecture, Node, Operation,
    OperationSite, Parameter,
};

use crate::{
    client::Qcs,
    python::{errors, py_function_sync_async},
    qpu::get_isa,
};

#[pymodule]
#[pyo3(name = "isa", module = "qcs_sdk.qpu", submodule)]
pub(crate) fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add(
        "SerializeISAError",
        py.get_type::<errors::SerializeISAError>(),
    )?;
    m.add("GetISAError", py.get_type::<errors::GetISAError>())?;

    m.add_class::<PyFamily>()?;
    m.add_class::<PyEdge>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyArchitecture>()?;
    m.add_class::<PyCharacteristic>()?;
    m.add_class::<PyParameter>()?;
    m.add_class::<PyOperationSite>()?;
    m.add_class::<PyOperation>()?;
    m.add_class::<PyInstructionSetArchitecture>()?;

    m.add_function(wrap_pyfunction!(py_get_instruction_set_architecture, m)?)?;
    m.add_function(wrap_pyfunction!(
        py_get_instruction_set_architecture_async,
        m
    )?)?;

    Ok(())
}

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Family", module = "qcs_sdk.qpu.isa")]
struct PyFamily(Family);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Edge", module = "qcs_sdk.qpu.isa")]
struct PyEdge(Edge);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Node", module = "qcs_sdk.qpu.isa")]
struct PyNode(Node);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Architecture", module = "qcs_sdk.qpu.isa")]
struct PyArchitecture(Architecture);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Characteristic", module = "qcs_sdk.qpu.isa")]
struct PyCharacteristic(Characteristic);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Parameter", module = "qcs_sdk.qpu.isa")]
struct PyParameter(Parameter);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "OperationSite", module = "qcs_sdk.qpu.isa")]
struct PyOperationSite(OperationSite);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Operation", module = "qcs_sdk.qpu.isa")]
struct PyOperation(Operation);

#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "InstructionSetArchitecture", module = "qcs_sdk.qpu.isa")]
struct PyInstructionSetArchitecture(InstructionSetArchitecture);

#[cfg(feature = "python")]
#[derive(Debug, thiserror::Error)]
#[error("Failed to serialize instruction set architecture: {0}")]
pub struct SerializeIsaError(#[from] serde_json::Error);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyInstructionSetArchitecture {
    #[staticmethod]
    pub fn from_raw(json: String) -> Result<Self, SerializeIsaError> {
        Ok(Self(serde_json::from_str(&json)?))
    }

    #[pyo3(signature = (pretty = false))]
    pub fn json(&self, pretty: bool) -> Result<String, SerializeIsaError> {
        let data = {
            if pretty {
                serde_json::to_string_pretty(&self.0)
            } else {
                serde_json::to_string(&self.0)
            }
        }?;
        Ok(data)
    }
}

py_function_sync_async! {
    #[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
    #[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk.qpu.isa"))]
    #[pyfunction]
    #[pyo3(signature = (quantum_processor_id, client = None))]
    async fn get_instruction_set_architecture(
        quantum_processor_id: String,
        client: Option<Qcs>,
    ) -> PyResult<PyInstructionSetArchitecture> {
        let client = client.unwrap_or_else(Qcs::load);

        get_isa(&quantum_processor_id, &client)
            .await
            .map(|isa| PyInstructionSetArchitecture(isa))
            .map_err(Into::into)
    }
}
