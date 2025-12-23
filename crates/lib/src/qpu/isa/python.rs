use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, py_function_sync_async};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyfunction, gen_stub_pymethods};

use qcs_api_client_openapi::models::{
    Architecture, Characteristic, Edge, Family, InstructionSetArchitecture, Node, Operation,
    OperationSite, Parameter,
};

use crate::{client::Qcs, python::errors, qpu::get_isa};

// #[pyo3(name = "isa", module = "qcs_sdk.qpu", submodule)]
create_init_submodule! {
    classes: [
        PyFamily,
        PyEdge,
        PyNode,
        PyArchitecture,
        PyCharacteristic,
        PyParameter,
        PyOperationSite,
        PyOperation,
        PyInstructionSetArchitecture
    ],
    errors: [ errors::SerializeISAError, errors::GetISAError ],
    funcs: [ py_get_instruction_set_architecture, py_get_instruction_set_architecture_async ],
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
pub(crate) struct PyInstructionSetArchitecture(pub InstructionSetArchitecture);

#[cfg(feature = "python")]
#[derive(Debug, thiserror::Error)]
#[error("Failed to serialize instruction set architecture: {0}")]
pub struct SerializeIsaError(#[from] serde_json::Error);

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyInstructionSetArchitecture {
    #[staticmethod]
    pub(crate) fn from_raw(json: String) -> Result<Self, SerializeIsaError> {
        Ok(Self(serde_json::from_str(&json)?))
    }

    #[pyo3(signature = (pretty = false))]
    pub(crate) fn json(&self, pretty: bool) -> Result<String, SerializeIsaError> {
        let data = {
            if pretty {
                serde_json::to_string_pretty(&self.0)
            } else {
                serde_json::to_string(&self.0)
            }
        }?;
        Ok(data)
    }

    #[getter]
    fn architecture(&self) -> PyArchitecture {
        PyArchitecture(*self.0.architecture.clone())
    }

    #[getter]
    fn benchmarks(&self) -> Vec<PyOperation> {
        self.0.benchmarks.iter().cloned().map(PyOperation).collect()
    }

    #[setter]
    fn set_benchmarks(&mut self, benchmarks: Vec<PyOperation>) {
        self.0.benchmarks = benchmarks.into_iter().map(|op| op.0).collect();
    }

    #[getter]
    fn instructions(&self) -> Vec<PyOperation> {
        self.0
            .instructions
            .iter()
            .cloned()
            .map(PyOperation)
            .collect()
    }

    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyOperation {
    #[getter]
    fn characteristics(&self) -> Vec<PyCharacteristic> {
        self.0
            .characteristics
            .iter()
            .cloned()
            .map(PyCharacteristic)
            .collect()
    }

    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    #[getter]
    fn node_count(&self) -> Option<i64> {
        self.0.node_count
    }

    #[getter]
    fn parameters(&self) -> Vec<PyParameter> {
        self.0.parameters.iter().cloned().map(PyParameter).collect()
    }

    #[getter]
    fn sites(&self) -> Vec<PyOperationSite> {
        self.0.sites.iter().cloned().map(PyOperationSite).collect()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyOperationSite {
    #[getter]
    fn characteristics(&self) -> Vec<PyCharacteristic> {
        self.0
            .characteristics
            .iter()
            .cloned()
            .map(PyCharacteristic)
            .collect()
    }

    #[getter]
    fn node_ids(&self) -> Vec<i64> {
        self.0.node_ids.clone()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyParameter {
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyCharacteristic {
    #[getter]
    fn error(&self) -> Option<f64> {
        self.0.error
    }

    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    #[getter]
    fn node_ids(&self) -> Option<Vec<i64>> {
        self.0.node_ids.clone()
    }

    #[getter]
    fn parameter_values(&self) -> Option<Vec<f64>> {
        self.0.parameter_values.clone()
    }

    #[getter]
    fn timestamp(&self) -> &str {
        &self.0.timestamp
    }

    #[getter]
    fn value(&self) -> f64 {
        self.0.value
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyArchitecture {
    #[getter]
    fn edges(&self) -> Vec<PyEdge> {
        self.0.edges.iter().cloned().map(PyEdge).collect()
    }

    #[getter]
    fn family(&self) -> Option<PyFamily> {
        self.0.family.clone().map(|f| PyFamily(*f))
    }

    #[getter]
    fn nodes(&self) -> Vec<PyNode> {
        self.0.nodes.iter().cloned().map(PyNode).collect()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyNode {
    #[getter]
    fn node_id(&self) -> i64 {
        self.0.node_id
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyEdge {
    #[getter]
    fn node_ids(&self) -> Vec<i64> {
        self.0.node_ids.clone()
    }
}

py_function_sync_async! {
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
            .map(PyInstructionSetArchitecture)
            .map_err(Into::into)
    }
}
