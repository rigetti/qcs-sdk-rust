from enum import Enum
from typing import List, Sequence, Optional, final

from qcs_sdk.client import QCSClient

class SerializeISAError(ValueError):
    """If an ``InstructionSetArchitecture`` could not be serialized or deserialized correctly."""

    ...

class GetISAError(RuntimeError):
    """If there was an issue fetching ``InstructionSetArchitecture`` from the QCS API."""

    ...

@final
class Family:
    """
    The architecture family identifier of an ``InstructionSetArchitecture``.

    Value "Full"  implies that each node is connected to every other (fully-connected architecture).
    """
    def is_ankaa(self) -> bool: ...
    def is_aspen(self) -> bool: ...
    def is_full(self) -> bool: ...
    def is_unknown(self) -> bool: ...
    def is_none(self) -> bool: ...
    def as_unknown(self) -> Optional[str]: ...
    def to_unknown(self) -> str: ...
    @staticmethod
    def from_unknown(inner: str) -> "Family": ...
    def inner(self) -> str: ...
    @staticmethod
    def new_ankaa() -> "Family": ...
    @staticmethod
    def new_aspen() -> "Family": ...
    @staticmethod
    def new_full() -> "Family": ...
    @staticmethod
    def new_none() -> "Family": ...

@final
class Node:
    """
    A logical node in the quantum processor's architecture.

    The existence of a node in the ISA ``Architecture`` does not necessarily mean that a given 1Q
    operation will be available on the node. This information is conveyed by the presence of the
    specific `node_id` in instances of ``Instruction``.
    """

    @property
    def node_id(self) -> int:
        """
        An integer id assigned to the computational node.
        The ids may not be contiguous and will be assigned based on the architecture family.
        """
        ...
    @node_id.setter
    def node_id(self, value: int): ...

@final
class Edge:
    """
    A degree-two logical connection in the quantum processor's architecture.

    The existence of an edge in the ISA ``Architecture`` does not necessarily mean that a given 2Q
    operation will be available on the edge. This information is conveyed by the presence of the
    two `node_id` values in instances of ``Instruction``.

    Note that edges are undirected in this model. Thus edge :math:`(a, b)` is equivalent to edge
    :math:`(b, a)`.
    """

    @property
    def node_ids(self) -> List[int]:
        """
        The integer ids of the computational nodes at the two ends of the edge.
        Order is not important; an architecture edge is treated as undirected.
        """
        ...
    @node_ids.setter
    def node_ids(self, value: List[int]): ...

@final
class Parameter:
    """A parameter to an operation."""

    @property
    def name(self) -> str:
        """The name of the parameter, such as the name of a mathematical symbol."""
        ...
    @name.setter
    def name(self, value: str): ...

@final
class Characteristic:
    """A measured characteristic of an operation."""

    @property
    def name(self) -> str:
        """The name of the characteristic"""
        ...
    @name.setter
    def name(self, value: str): ...
    @property
    def value(self) -> float:
        """The characteristic value measured."""
        ...
    @value.setter
    def value(self, value: float): ...
    @property
    def error(self) -> Optional[float]:
        """The error in the characteristic value, or None if otherwise."""
        ...
    @error.setter
    def error(self, value: Optional[float]): ...
    @property
    def node_ids(self) -> Optional[List[int]]:
        """
        The list of architecture node ids for the site where the characteristic is
        measured, if that is different from the site of the enclosing operation.
        `None` if it is the same. The order of this or the enclosing node ids obey
        the definition of node symmetry from the enclosing operation.
        """
        ...
    @node_ids.setter
    def node_ids(self, value: Optional[Sequence[int]]): ...
    @property
    def timestamp(self) -> str:
        """ISO8601 date and time at which the characteristic was measured."""
        ...
    @timestamp.setter
    def timestamp(self, value: str): ...
    @property
    def parameter_values(self) -> Optional[List[float]]:
        """
        The optional ordered list of parameter values used to generate the characteristic.
        he order matches the parameters in the enclosing operation, and so the lengths of
        these two lists must match.
        """
        ...
    @parameter_values.setter
    def parameter_values(self, value: Optional[Sequence[float]]): ...

@final
class OperationSite:
    """A site for an operation, with its site-dependent characteristics."""

    @property
    def node_ids(self) -> List[int]:
        """
        The list of architecture node ids for the site. The order of these node ids
        obey the definition of node symmetry from the enclosing operation.
        """
        ...
    @node_ids.setter
    def node_ids(self, value: Sequence[int]): ...
    @property
    def characteristics(self) -> List[Characteristic]:
        """The list of site-dependent characteristics of this operation."""
        ...
    @characteristics.setter
    def characteristics(self, value: Sequence[Characteristic]): ...

@final
class Operation:
    """An operation, with its sites and site-independent characteristics."""

    @property
    def name(self) -> str:
        """The name of the operation."""
        ...
    @name.setter
    def name(self, value: str): ...
    @property
    def node_count(self) -> Optional[int]:
        """The number of nodes that this operation applies to. None if unspecified."""
        ...
    @node_count.setter
    def node_count(self, value: Optional[int]): ...
    @property
    def parameters(self) -> List[Parameter]:
        """The list of parameters. Each parameter must be uniquely named. May be empty."""
        ...
    @parameters.setter
    def parameters(self, value: Sequence[Parameter]): ...
    @property
    def sites(self) -> List[OperationSite]:
        """
        The list of sites at which this operation can be applied,
        together with its site-dependent characteristics.
        """
        ...
    @sites.setter
    def sites(self, value: Sequence[OperationSite]): ...
    @property
    def characteristics(self) -> List[Characteristic]:
        """The list of site-independent characteristics of this operation."""
        ...
    @characteristics.setter
    def characteristics(self, value: Sequence[Characteristic]): ...

@final
class Architecture:
    """
    Represents the logical underlying architecture of a quantum processor.

    The architecture is defined in detail by the nodes and edges that constitute the quantum
    processor. This defines the set of all nodes that could be operated upon, and indicates to
    some approximation their physical layout. The main purpose of this is to support geometry
    calculations that are independent of the available operations, and rendering ISA-based
    information. Architecture layouts are defined by the `family`, as follows.

    The "Aspen" family of quantum processor indicates a 2D planar grid layout of octagon unit
    cells. The `node_id` in this architecture is computed as :math:`100 p_y + 10 p_x + p_u` where
    :math:`p_y` is the zero-based Y position in the unit cell grid, :math:`p_x` is the zero-based
    X position in the unit cell grid, and :math:`p_u` is the zero-based position in the octagon
    unit cell and always ranges from 0 to 7.

    The "Ankaa" architecture is based on a grid topology; having, in "vertical" orientation,
    qubits numbered starting from 0 at the top-left and increasing from left to right,
    then top to bottom, so the final qubit is in the bottom-right. Each qubit is connected
    with a tunable coupler to their direct vertical and horizontal neighbors, producing an edge.
    Edges are ordered top-left to bottom-right in this orientation as well, with horizontal rows
    alternating with vertical rows. Ankaa chips are, in vertical orientation,
    7 qubits wide and 12 tall. This architecture may also be presented in "landscape"
    orientation, which is a simple 90-degree clockwise rotation of the vertical orientation.

    Note that the operations that are actually available are defined entirely by ``Operation``
    instances. The presence of a node or edge in the ``Architecture`` model provides no guarantee
    that any 1Q or 2Q operation will be available to users writing QUIL programs.
    """

    @property
    def family(self) -> Family:
        """The architecture family. The nodes and edges conform to this family."""
        ...
    @family.setter
    def family(self, value: Family): ...
    @property
    def nodes(self) -> List[Node]:
        """A list of all computational nodes in the instruction set architecture."""
        ...
    @nodes.setter
    def nodes(self, value: Sequence[Node]): ...
    @property
    def edges(self) -> List[Edge]:
        """A list of all computational edges in the instruction set architecture."""
        ...
    @edges.setter
    def edges(self, value: Sequence[Edge]): ...

@final
class InstructionSetArchitecture:
    """
    The native instruction set architecture (ISA) of a quantum processor, annotated with characteristics.

    The operations described by the `instructions` field are named by their QUIL instruction name,
    while the operation described by the `benchmarks` field are named by their benchmark routine
    and are a future extension point.

    The characteristics that annotate both instructions and benchmarks assist the user to generate
    the best native QUIL program for a desired task, and so are provided as part of the native ISA.
    """

    @property
    def name(self) -> str:
        """The name of the quantum processor."""
        ...
    @name.setter
    def name(self, value: str): ...
    @property
    def architecture(self) -> Architecture:
        """The architecture of the quantum processor."""
        ...
    @architecture.setter
    def architecture(self, value: Architecture): ...
    @property
    def instructions(self) -> List[Operation]:
        """The list of native QUIL instructions supported by the quantum processor."""
        ...
    @instructions.setter
    def instructions(self, value: Sequence[Operation]): ...
    @property
    def benchmarks(self) -> List[Operation]:
        """The list of benchmarks that have characterized the quantum processor."""
        ...
    @benchmarks.setter
    def benchmarks(self, value: Sequence[Operation]): ...
    @staticmethod
    def from_raw(json: str) -> "InstructionSetArchitecture":
        """
        Deserialize an ``InstructionSetArchitecture`` from a json representation.

        :param value: The json-serialized ``InstructionSetArchitecture`` to deserialize.

        :raises SerializeISAError: If the input string was not deserialized correctly.
        """
        ...
    def json(self, pretty: bool = False) -> str:
        """
        Serialize the ``InstructionSetArchitecture`` to a json string, optionally pretty-printed.

        :param pretty: If the json output should be pretty-printed with newlines and indents.

        :raises SerializeISAError: If the input string was not serialized correctly.
        """
        ...

def get_instruction_set_architecture(
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
) -> InstructionSetArchitecture:
    """
    Fetch the ``InstructionSetArchitecture`` (ISA) for a given `quantum_processor_id` from the QCS API.

    :param quantum_processor_id: The ID of the quantum processor
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :raises LoadClientError: If ``client`` was not provided to the function, and failed to load internally.
    :raises GetISAError: If there was an issue fetching the ISA from the QCS API.
    """
    ...

async def get_instruction_set_architecture_async(
    quantum_processor_id: str, client: Optional[QCSClient] = None
) -> InstructionSetArchitecture:
    """
    Fetch the ``InstructionSetArchitecture`` (ISA) for a given `quantum_processor_id` from the QCS API.
    (async analog of ``get_instruction_set_architecture``)

    :param quantum_processor_id: The ID of the quantum processor
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :raises LoadClientError: If ``client`` was not provided to the function, and failed to load internally.
    :raises GetISAError: If there was an issue fetching the ISA from the QCS API.
    """
    ...
