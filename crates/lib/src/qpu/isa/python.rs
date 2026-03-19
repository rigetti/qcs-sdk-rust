use pyo3::prelude::*;
use rigetti_pyo3::{create_init_submodule, impl_repr, py_function_sync_async};
use serde::{Deserialize, Serialize};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::{
    derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pyfunction, gen_stub_pymethods},
    impl_stub_type,
};

use qcs_api_client_openapi::models;

use crate::{client::Qcs, python::errors, qpu::get_isa};

create_init_submodule! {
    classes: [
        Architecture,
        Characteristic,
        Edge,
        Family,
        InstructionSetArchitecture,
        Node,
        Operation,
        OperationSite,
        Parameter
    ],
    errors: [ errors::SerializeISAError, errors::GetISAError ],
    funcs: [ py_get_instruction_set_architecture, py_get_instruction_set_architecture_async ],
}

impl_repr!(Architecture);
impl_repr!(Characteristic);
impl_repr!(Edge);
impl_repr!(InstructionSetArchitecture);
impl_repr!(Node);
impl_repr!(Operation);
impl_repr!(OperationSite);
impl_repr!(Parameter);

/// The native instruction set architecture (ISA) of a quantum processor, annotated with characteristics.
///
/// The operations described by the `instructions` field are named by their QUIL instruction name,
/// while the operation described by the `benchmarks` field are named by their benchmark routine
/// and are a future extension point.
///
/// The characteristics that annotate both instructions and benchmarks assist the user to generate
/// the best native QUIL program for a desired task, and so are provided as part of the native ISA.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
pub(crate) struct InstructionSetArchitecture {
    architecture: Architecture,
    /// The list of benchmarks that have characterized the quantum processor.
    benchmarks: Vec<Operation>,
    /// The list of native QUIL instructions supported by the quantum processor.
    instructions: Vec<Operation>,
    /// The name of the quantum processor.
    name: String,
}

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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct Architecture {
    /// A list of all computational edges in the instruction set architecture.
    pub edges: Vec<Edge>,
    /// The architecture family. The nodes and edges conform to this family.
    pub family: Option<PyFamily>,
    /// A list of all computational nodes in the instruction set architecture.
    pub nodes: Vec<Node>,
}

/// The architecture family identifier of an ``InstructionSetArchitecture``.
///
/// Value 'NONE' implies the architecture has no specific layout topology.
/// Value 'FULL' implies that each node is connected to every other (a fully-connected architecture).
/// For other values based on deployed architecture layouts (e.g. `Aspen` and `Ankaa`),
/// refer to the architecture classes themselves for more details.
///
/// Note: Within an ``InstructionSetArchitecture``, the `family` may be one of these,
/// or may be a `str` for an unknown family, or may be `None` if the `family` is not specified.
/// The latter in particular is distinct from the `NONE` value within this enumeration.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass_enum)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "qcs_sdk.qpu.isa", eq, rename_all = "SCREAMING_SNAKE_CASE")
)]
enum Family {
    #[default]
    None,
    Full,
    Aspen,
    Ankaa,
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    pyo3::FromPyObject,
    pyo3::IntoPyObject,
)]
#[serde(untagged)]
enum PyFamily {
    #[pyo3(transparent)]
    Known(Family),
    #[pyo3(transparent)]
    Unknown(String),
}

#[cfg(feature = "stubs")]
impl_stub_type!(PyFamily = Family | String);

impl Default for PyFamily {
    fn default() -> Self {
        Self::Known(Family::default())
    }
}

impl<T: AsRef<models::Family>> From<T> for PyFamily {
    fn from(family: T) -> Self {
        match family.as_ref() {
            models::Family::None => PyFamily::Known(Family::None),
            models::Family::Full => PyFamily::Known(Family::Full),
            models::Family::Aspen => PyFamily::Known(Family::Aspen),
            models::Family::Ankaa => PyFamily::Known(Family::Ankaa),
            models::Family::Unknown(s) => PyFamily::Unknown(s.clone()),
        }
    }
}

impl From<PyFamily> for models::Family {
    fn from(family: PyFamily) -> Self {
        match family {
            PyFamily::Known(f) => f.into(),
            PyFamily::Unknown(s) => models::Family::Unknown(s),
        }
    }
}

impl From<Family> for models::Family {
    fn from(family: Family) -> Self {
        match family {
            Family::None => models::Family::None,
            Family::Full => models::Family::Full,
            Family::Aspen => models::Family::Aspen,
            Family::Ankaa => models::Family::Ankaa,
        }
    }
}

/// A degree-two logical connection in the quantum processor's architecture.
///
/// The existence of an edge in the ISA ``Architecture`` does not necessarily mean that a given 2Q
/// operation will be available on the edge. This information is conveyed by the presence of the
/// two `node_id` values in instances of ``Instruction``.
///
/// Note that edges are undirected in this model. Thus edge :math:`(a, b)` is equivalent to edge
/// :math:`(b, a)`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct Edge {
    /// The integer ids of the computational nodes at the two ends of the edge.
    /// Order is not important; an architecture edge is treated as undirected.
    pub node_ids: Vec<i64>,
}

/// A logical node in the quantum processor's architecture.
///
/// The existence of a node in the ISA ``Architecture`` does not necessarily mean that a given 1Q
/// operation will be available on the node. This information is conveyed by the presence of the
/// specific `node_id` in instances of ``Instruction``.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct Node {
    /// An integer id assigned to the computational node.
    ///
    /// The ids may not be contiguous and will be assigned based on the architecture family.
    pub node_id: i64,
}

/// A measured characteristic of an operation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct Characteristic {
    /// The error in the characteristic value, or None otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<f64>,
    /// The name of the characteristic.
    pub name: String,
    /// The list of architecture node ids for the site where the characteristic is measured, if that is different from the site of the enclosing operation. None if it is the same. The order of this or the enclosing node ids obey the definition of node symmetry from the enclosing operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<i64>>,
    /// The optional ordered list of parameter values used to generate the characteristic. The order matches the parameters in the enclosing operation, and so the lengths of these two lists must match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<Vec<f64>>,
    /// The date and time at which the characteristic was measured.
    pub timestamp: String,
    /// The characteristic value measured.
    pub value: f64,
}

/// A parameter to an operation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct Parameter {
    /// The name of the parameter, such as the name of a mathematical symbol.
    pub name: String,
}

/// A site for an operation, with its site-dependent characteristics.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct OperationSite {
    /// The list of site-dependent characteristics of this operation.
    pub characteristics: Vec<Characteristic>,
    /// The list of architecture node ids for the site.
    ///
    /// The order of these node ids obey the definition of node symmetry from the enclosing operation.
    pub node_ids: Vec<i64>,
}

/// An operation, with its sites and site-independent characteristics.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "stubs", gen_stub_pyclass)]
#[pyclass(module = "qcs_sdk.qpu.isa", eq, get_all, set_all)]
struct Operation {
    /// The list of site-independent characteristics of this operation.
    pub characteristics: Vec<Characteristic>,
    /// The name of the operation.
    pub name: String,
    /// The number of nodes that this operation applies to. None if unspecified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_count: Option<u64>,
    /// The list of parameters. Each parameter must be uniquely named. May be empty.
    pub parameters: Vec<Parameter>,
    /// The list of sites at which this operation can be applied, together with its site-dependent characteristics.
    pub sites: Vec<OperationSite>,
}

#[cfg(feature = "python")]
#[derive(Debug, thiserror::Error)]
#[error("Failed to serialize instruction set architecture: {0}")]
pub struct SerializeIsaError(#[from] serde_json::Error);

impl From<models::InstructionSetArchitecture> for InstructionSetArchitecture {
    fn from(isa: models::InstructionSetArchitecture) -> Self {
        Self {
            architecture: isa.architecture.into(),
            benchmarks: convert_vec(isa.benchmarks),
            instructions: convert_vec(isa.instructions),
            name: isa.name,
        }
    }
}

impl From<InstructionSetArchitecture> for models::InstructionSetArchitecture {
    fn from(isa: InstructionSetArchitecture) -> Self {
        Self {
            architecture: isa.architecture.into(),
            benchmarks: convert_vec(isa.benchmarks),
            instructions: convert_vec(isa.instructions),
            name: isa.name,
        }
    }
}

impl From<models::Architecture> for Architecture {
    fn from(arch: models::Architecture) -> Self {
        Self {
            edges: arch
                .edges
                .into_iter()
                .map(|e| Edge {
                    node_ids: e.node_ids,
                })
                .collect(),
            family: arch.family.map(Into::into),
            nodes: convert_vec(arch.nodes),
        }
    }
}

impl From<Architecture> for models::Architecture {
    fn from(arch: Architecture) -> Self {
        Self {
            edges: arch
                .edges
                .into_iter()
                .map(|e| models::Edge {
                    node_ids: e.node_ids,
                })
                .collect(),
            family: arch.family.map(Into::into).unwrap_or_default(),
            nodes: convert_vec(arch.nodes),
        }
    }
}

impl From<models::Edge> for Edge {
    fn from(edge: models::Edge) -> Self {
        Self {
            node_ids: edge.node_ids,
        }
    }
}

impl From<Edge> for models::Edge {
    fn from(edge: Edge) -> Self {
        Self {
            node_ids: edge.node_ids,
        }
    }
}

impl From<models::Node> for Node {
    fn from(node: models::Node) -> Self {
        Self {
            node_id: node.node_id,
        }
    }
}

impl From<Node> for models::Node {
    fn from(node: Node) -> Self {
        Self {
            node_id: node.node_id,
        }
    }
}

impl From<models::Characteristic> for Characteristic {
    fn from(characteristic: models::Characteristic) -> Self {
        Self {
            error: characteristic.error,
            name: characteristic.name,
            node_ids: characteristic.node_ids,
            parameter_values: characteristic.parameter_values,
            timestamp: characteristic.timestamp,
            value: characteristic.value,
        }
    }
}

impl From<Characteristic> for models::Characteristic {
    fn from(characteristic: Characteristic) -> Self {
        Self {
            error: characteristic.error,
            name: characteristic.name,
            node_ids: characteristic.node_ids,
            parameter_values: characteristic.parameter_values,
            timestamp: characteristic.timestamp,
            value: characteristic.value,
        }
    }
}

impl From<models::Parameter> for Parameter {
    fn from(parameter: models::Parameter) -> Self {
        Self {
            name: parameter.name,
        }
    }
}

impl From<Parameter> for models::Parameter {
    fn from(parameter: Parameter) -> Self {
        Self {
            name: parameter.name,
        }
    }
}

impl From<models::OperationSite> for OperationSite {
    fn from(site: models::OperationSite) -> Self {
        Self {
            characteristics: convert_vec(site.characteristics),
            node_ids: site.node_ids,
        }
    }
}

impl From<OperationSite> for models::OperationSite {
    fn from(site: OperationSite) -> Self {
        Self {
            characteristics: convert_vec(site.characteristics),
            node_ids: site.node_ids,
        }
    }
}

impl From<models::Operation> for Operation {
    fn from(operation: models::Operation) -> Self {
        Self {
            characteristics: convert_vec(operation.characteristics),
            name: operation.name,
            node_count: operation.node_count,
            parameters: convert_vec(operation.parameters),
            sites: convert_vec(operation.sites),
        }
    }
}

impl From<Operation> for models::Operation {
    fn from(operation: Operation) -> Self {
        Self {
            characteristics: convert_vec(operation.characteristics),
            name: operation.name,
            node_count: operation.node_count,
            parameters: convert_vec(operation.parameters),
            sites: convert_vec(operation.sites),
        }
    }
}

fn convert_vec<T, U>(vec: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    vec.into_iter().map(Into::into).collect()
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl InstructionSetArchitecture {
    #[new]
    fn __new__(
        architecture: Architecture,
        benchmarks: Vec<Operation>,
        instructions: Vec<Operation>,
        name: String,
    ) -> Self {
        Self {
            architecture,
            benchmarks,
            instructions,
            name,
        }
    }

    /// Deserialize an `InstructionSetArchitecture` from a json representation.
    ///
    /// # Errors
    ///
    /// Returns [`SerializeIsaError`] if the input string was not deserialized correctly.
    #[staticmethod]
    pub(crate) fn from_raw(json: &str) -> Result<Self, SerializeIsaError> {
        Ok(serde_json::from_str(json)?)
    }

    /// Serialize the `InstructionSetArchitecture` to a json string, optionally pretty-printed.
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
                serde_json::to_string_pretty(&self)
            } else {
                serde_json::to_string(&self)
            }
        }?;
        Ok(data)
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Operation {
    #[new]
    /// An operation, with its sites and site-independent characteristics.
    fn __new__(
        characteristics: Vec<Characteristic>,
        name: String,
        parameters: Vec<Parameter>,
        sites: Vec<OperationSite>,
    ) -> Self {
        Self {
            characteristics,
            name,
            node_count: None,
            parameters,
            sites,
        }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl OperationSite {
    #[new]
    /// A site for an operation, with its site-dependent characteristics.
    fn __new__(characteristics: Vec<Characteristic>, node_ids: Vec<i64>) -> Self {
        Self {
            characteristics,
            node_ids,
        }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Parameter {
    #[new]
    fn __new__(name: String) -> Self {
        Self { name }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Characteristic {
    #[new]
    /// A measured characteristic of an operation.
    fn __new__(name: String, timestamp: String, value: f64) -> Self {
        Self {
            error: None,
            name,
            node_ids: None,
            parameter_values: None,
            timestamp,
            value,
        }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Architecture {
    #[new]
    fn __new__(edges: Vec<Edge>, family: Option<PyFamily>, nodes: Vec<Node>) -> Self {
        Self {
            edges,
            family,
            nodes,
        }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Node {
    #[new]
    fn __new__(node_id: i64) -> Self {
        Self { node_id }
    }
}

#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl Edge {
    #[new]
    fn __new__(node_ids: Vec<i64>) -> Self {
        Self { node_ids }
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
    ) -> PyResult<InstructionSetArchitecture> {
        let client = client.unwrap_or_else(Qcs::load);

        get_isa(&quantum_processor_id, &client)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }
}
