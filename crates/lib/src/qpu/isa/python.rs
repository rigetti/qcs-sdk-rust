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

/// The native instruction set architecture (ISA) of a quantum processor, annotated with characteristics.
///
/// The operations described by the `instructions` field are named by their QUIL instruction name,
/// while the operation described by the `benchmarks` field are named by their benchmark routine
/// and are a future extension point.
///
/// The characteristics that annotate both instructions and benchmarks assist the user to generate
/// the best native QUIL program for a desired task, and so are provided as part of the native ISA.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "InstructionSetArchitecture", module = "qcs_sdk.qpu.isa")]
pub(crate) struct PyInstructionSetArchitecture(pub InstructionSetArchitecture);

/// Represents the logical underlying architecture of a quantum processor.
///
/// The architecture is defined in detail by the nodes and edges that constitute the quantum
/// processor. This defines the set of all nodes that could be operated upon, and indicates to
/// some approximation their physical layout. The main purpose of this is to support geometry
/// calculations that are independent of the available operations, and rendering ISA-based
/// information. Architecture layouts are defined by the `family`, as follows.
///
/// The "Aspen" family of quantum processor indicates a 2D planar grid layout of octagon unit
/// cells. The `node_id` in this architecture is computed as :math:`100 p_y + 10 p_x + p_u` where
/// :math:`p_y` is the zero-based Y position in the unit cell grid, :math:`p_x` is the zero-based
/// X position in the unit cell grid, and :math:`p_u` is the zero-based position in the octagon
/// unit cell and always ranges from 0 to 7.
///
/// The "Ankaa" architecture is based on a grid topology; having, in "vertical" orientation,
/// qubits numbered starting from 0 at the top-left and increasing from left to right,
/// then top to bottom, so the final qubit is in the bottom-right. Each qubit is connected
/// with a tunable coupler to their direct vertical and horizontal neighbors, producing an edge.
/// Edges are ordered top-left to bottom-right in this orientation as well, with horizontal rows
/// alternating with vertical rows. Ankaa chips are, in vertical orientation,
/// 7 qubits wide and 12 tall. This architecture may also be presented in "landscape"
/// orientation, which is a simple 90-degree clockwise rotation of the vertical orientation.
///
/// Note that the operations that are actually available are defined entirely by ``Operation``
/// instances. The presence of a node or edge in the ``Architecture`` model provides no guarantee
/// that any 1Q or 2Q operation will be available to users writing QUIL programs.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Architecture", module = "qcs_sdk.qpu.isa")]
struct PyArchitecture(Architecture);

/// The architecture family identifier of an ``InstructionSetArchitecture``.
///
/// Value "Full" implies that each node is connected to every other (fully-connected architecture).
#[derive(Clone)]
#[expect(dead_code)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Family", module = "qcs_sdk.qpu.isa")]
struct PyFamily(Family);

/// A degree-two logical connection in the quantum processor's architecture.
///
/// The existence of an edge in the ISA ``Architecture`` does not necessarily mean that a given 2Q
/// operation will be available on the edge. This information is conveyed by the presence of the
/// two `node_id` values in instances of ``Instruction``.
///
/// Note that edges are undirected in this model. Thus edge :math:`(a, b)` is equivalent to edge
/// :math:`(b, a)`.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Edge", module = "qcs_sdk.qpu.isa")]
struct PyEdge(Edge);

/// A logical node in the quantum processor's architecture.
///
/// The existence of a node in the ISA ``Architecture`` does not necessarily mean that a given 1Q
/// operation will be available on the node. This information is conveyed by the presence of the
/// specific `node_id` in instances of ``Instruction``.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Node", module = "qcs_sdk.qpu.isa")]
struct PyNode(Node);

/// A measured characteristic of an operation.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Characteristic", module = "qcs_sdk.qpu.isa")]
struct PyCharacteristic(Characteristic);

/// A parameter to an operation.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Parameter", module = "qcs_sdk.qpu.isa")]
struct PyParameter(Parameter);

/// A site for an operation, with its site-dependent characteristics.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "OperationSite", module = "qcs_sdk.qpu.isa")]
struct PyOperationSite(OperationSite);

/// An operation, with its sites and site-independent characteristics.
#[derive(Clone)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(name = "Operation", module = "qcs_sdk.qpu.isa")]
struct PyOperation(Operation);

#[cfg(feature = "python")]
#[derive(Debug, thiserror::Error)]
#[error("Failed to serialize instruction set architecture: {0}")]
pub struct SerializeIsaError(#[from] serde_json::Error);

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyInstructionSetArchitecture {
    /// Deserialize an [`InstructionSetArchitecture`] from a json representation.
    ///
    /// # Errors
    ///
    /// Returns `[SerializeIsaError`] if the input string was not deserialized correctly.
    #[staticmethod]
    pub(crate) fn from_raw(json: &str) -> Result<Self, SerializeIsaError> {
        Ok(Self(serde_json::from_str(json)?))
    }

    /// Serialize the ``InstructionSetArchitecture`` to a json string, optionally pretty-printed.
    ///
    /// If `pretty` is true, the json output should be pretty-printed with newlines and indents.
    ///
    /// # Errors 
    ///
    /// Returns [`SerializeIsaError`] if the ISA could not be serialized. 
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

    /// The architecture of the quantum processor.
    #[getter]
    fn architecture(&self) -> PyArchitecture {
        PyArchitecture(*self.0.architecture.clone())
    }

    #[setter]
    fn set_architecture(&mut self, architecture: PyArchitecture) {
        *self.0.architecture = architecture.0;
    }

    /// The list of benchmarks that have characterized the quantum processor.
    #[getter]
    fn benchmarks(&self) -> Vec<PyOperation> {
        self.0.benchmarks.iter().cloned().map(PyOperation).collect()
    }

    #[setter]
    fn set_benchmarks(&mut self, benchmarks: Vec<PyOperation>) {
        self.0.benchmarks = benchmarks.into_iter().map(|op| op.0).collect();
    }

    /// The list of native QUIL instructions supported by the quantum processor.
    #[getter]
    fn instructions(&self) -> Vec<PyOperation> {
        self.0
            .instructions
            .iter()
            .cloned()
            .map(PyOperation)
            .collect()
    }
    
    #[setter]
    fn set_instructions(&mut self, instructions: Vec<PyOperation>) {
        self.0.instructions = instructions.into_iter().map(|op| op.0).collect();
    }

    /// The name of the quantum processor.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    #[setter]
    fn set_name(&mut self, name: String) {
        self.0.name = name;
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyOperation {
    /// The list of site-independent characteristics of this operation.
    #[getter]
    fn characteristics(&self) -> Vec<PyCharacteristic> {
        self.0
            .characteristics
            .iter()
            .cloned()
            .map(PyCharacteristic)
            .collect()
    }

    /// The name of the operation.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    /// The number of nodes that this operation applies to. None if unspecified.
    #[getter]
    fn node_count(&self) -> Option<i64> {
        self.0.node_count
    }

    /// The list of parameters. Each parameter must be uniquely named. May be empty.
    #[getter]
    fn parameters(&self) -> Vec<PyParameter> {
        self.0.parameters.iter().cloned().map(PyParameter).collect()
    }

    /// The list of sites at which this operation can be applied,
    /// together with its site-dependent characteristics.
    #[getter]
    fn sites(&self) -> Vec<PyOperationSite> {
        self.0.sites.iter().cloned().map(PyOperationSite).collect()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyOperationSite {
    /// The list of site-dependent characteristics of this operation.
    #[getter]
    fn characteristics(&self) -> Vec<PyCharacteristic> {
        self.0
            .characteristics
            .iter()
            .cloned()
            .map(PyCharacteristic)
            .collect()
    }

    /// The list of architecture node ids for the site. The order of these node ids
    /// obey the definition of node symmetry from the enclosing operation.
    #[getter]
    fn node_ids(&self) -> Vec<i64> {
        self.0.node_ids.clone()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyParameter {
    /// The name of the parameter, such as the name of a mathematical symbol.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyCharacteristic {
    /// The error in the characteristic value, or None if otherwise.
    #[getter]
    fn error(&self) -> Option<f64> {
        self.0.error
    }

    /// The name of the characteristic.
    #[getter]
    fn name(&self) -> &str {
        &self.0.name
    }

    /// The list of architecture node ids for the site where the characteristic is
    /// measured, if that is different from the site of the enclosing operation.
    /// `None` if it is the same. The order of this or the enclosing node ids obey
    /// the definition of node symmetry from the enclosing operation.
    #[getter]
    fn node_ids(&self) -> Option<Vec<i64>> {
        self.0.node_ids.clone()
    }

    /// The optional ordered list of parameter values used to generate the characteristic.
    /// The order matches the parameters in the enclosing operation, and so the lengths of
    /// these two lists must match.
    #[getter]
    fn parameter_values(&self) -> Option<Vec<f64>> {
        self.0.parameter_values.clone()
    }

    /// ISO8601 date and time at which the characteristic was measured.
    #[getter]
    fn timestamp(&self) -> &str {
        &self.0.timestamp
    }

    /// The characteristic value measured.
    #[getter]
    fn value(&self) -> f64 {
        self.0.value
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyArchitecture {
    /// A list of all computational edges in the instruction set architecture.
    #[getter]
    fn edges(&self) -> Vec<PyEdge> {
        self.0.edges.iter().cloned().map(PyEdge).collect()
    }

    /// The architecture family. The nodes and edges conform to this family.
    #[getter]
    fn family(&self) -> Option<PyFamily> {
        self.0.family.clone().map(|f| PyFamily(*f))
    }

    /// A list of all computational nodes in the instruction set architecture.
    #[getter]
    fn nodes(&self) -> Vec<PyNode> {
        self.0.nodes.iter().cloned().map(PyNode).collect()
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyNode {
    /// An integer id assigned to the computational node.
    /// The ids may not be contiguous and will be assigned based on the architecture family.
    #[getter]
    fn node_id(&self) -> i64 {
        self.0.node_id
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl PyEdge {
    /// The integer ids of the computational nodes at the two ends of the edge.
    /// Order is not important; an architecture edge is treated as undirected.
    #[getter]
    fn node_ids(&self) -> Vec<i64> {
        self.0.node_ids.clone()
    }
}

py_function_sync_async! {
    /// Fetch the ``InstructionSetArchitecture`` (ISA) for a given `quantum_processor_id` from the QCS API.
    ///
    /// :param quantum_processor_id: The ID of the quantum processor.
    /// :param client: The ``Qcs`` client to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    ///
    /// :raises LoadClientError: If ``client`` was not provided to the function, and failed to load internally.
    /// :raises GetISAError: If there was an issue fetching the ISA from the QCS API.
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
