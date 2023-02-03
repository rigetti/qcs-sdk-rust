use qcs_api_client_openapi::models::{
    Architecture1, Characteristic, Edge, Family, InstructionSetArchitecture, Node, Operation,
    OperationSite, Parameter,
};
use rigetti_pyo3::{
    create_init_submodule, py_wrap_data_struct, py_wrap_error, py_wrap_simple_enum,
    pyo3::{
        exceptions::{PyRuntimeError, PyValueError},
        prelude::*,
        types::{PyFloat, PyInt, PyList, PyString},
        Py, Python,
    },
    wrap_error, ToPythonError,
};

use qcs::qpu::get_isa;

use crate::qpu::client::PyQcsClient;

create_init_submodule! {
    classes: [
        PyFamily,
        PyEdge,
        PyNode,
        PyArchitecture1,
        PyCharacteristic,
        PyParameter,
        PyOperationSite,
        PyOperation,
        PyInstructionSetArchitecture
    ],
    errors: [
        QcsIsaSerializationError,
        QcsIsaError
    ],
    funcs: [
        py_get_instruction_set_architecture
    ],
}

wrap_error!(IsaSerializationError(serde_json::Error));
py_wrap_error!(
    api,
    IsaSerializationError,
    QcsIsaSerializationError,
    PyValueError
);

wrap_error!(IsaError(qcs::qpu::IsaError));
py_wrap_error!(api, IsaError, QcsIsaError, PyRuntimeError);

py_wrap_simple_enum! {
    PyFamily(Family) as "Family" {
        None as NONE,
        Full as Full,
        Aspen as Aspen,
        Ankaa as Ankaa
    }
}

py_wrap_data_struct! {
    PyEdge(Edge) as "Edge" {
        node_ids: Vec<i32> => Py<PyList>
    }
}

py_wrap_data_struct! {
    PyNode(Node) as "Node" {
        node_id: i32 => Py<PyInt>
    }
}

py_wrap_data_struct! {
    PyArchitecture1(Architecture1) as "Architecture" {
        edges: Vec<Edge> => Vec<PyEdge>,
        family: Option<Box<Family>> => Option<PyFamily>,
        nodes: Vec<Node> => Vec<PyNode>
    }
}

py_wrap_data_struct! {
    PyCharacteristic(Characteristic) as "Characteristic" {
        error: Option<f64> => Option<Py<PyFloat>>,
        name: String => Py<PyString>,
        node_ids: Option<Vec<i32>> => Option<Py<PyList>>,
        parameter_values: Option<Vec<f64>> => Option<Py<PyList>>,
        timestamp: String => Py<PyString>,
        value: f64 => Py<PyFloat>
    }
}

py_wrap_data_struct! {
    PyParameter(Parameter) as "Parameter" {
        name: String => Py<PyString>
    }
}

py_wrap_data_struct! {
    PyOperationSite(OperationSite) as "OperationSite" {
        characteristics: Vec<Characteristic> => Vec<PyCharacteristic>,
        node_ids: Vec<i32> => Py<PyList>
    }
}

py_wrap_data_struct! {
    PyOperation(Operation) as "Operation" {
        characteristics: Vec<Characteristic> => Vec<PyCharacteristic>,
        name: String => Py<PyString>,
        node_count: Option<i32> => Option<Py<PyInt>>,
        parameters: Vec<Parameter> => Vec<PyParameter>,
        sites: Vec<OperationSite> => Vec<PyOperationSite>
    }
}

py_wrap_data_struct! {
    PyInstructionSetArchitecture(InstructionSetArchitecture) as "InstructionSetArchitecture" {
        architecture: Box<Architecture1> => PyArchitecture1,
        benchmarks: Vec<Operation> => Vec<PyOperation>,
        instructions: Vec<Operation> => Vec<PyOperation>,
        name: String => Py<PyString>
    }
}

#[pymethods]
impl PyInstructionSetArchitecture {
    #[staticmethod]
    pub fn from_raw(json: String) -> PyResult<Self> {
        let isa = serde_json::from_str(&json)
            .map_err(IsaSerializationError::from)
            .map_err(IsaSerializationError::to_py_err)?;
        Ok(Self(isa))
    }

    #[args(pretty = "false")]
    pub fn json(&self, pretty: bool) -> PyResult<String> {
        let data = {
            if pretty {
                serde_json::to_string_pretty(&self.0)
            } else {
                serde_json::to_string(&self.0)
            }
        }
        .map_err(IsaSerializationError::from)
        .map_err(IsaSerializationError::to_py_err)?;
        Ok(data)
    }
}

#[pyfunction(client = "None")]
#[pyo3(name = "get_instruction_set_architecture")]
pub(crate) fn py_get_instruction_set_architecture(
    py: Python<'_>,
    quantum_processor_id: String,
    client: Option<PyQcsClient>,
) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let client = PyQcsClient::get_or_create_client(client).await?;

        get_isa(&quantum_processor_id, &client)
            .await
            .map(PyInstructionSetArchitecture::from)
            .map_err(IsaError::from)
            .map_err(IsaError::to_py_err)
    })
}
