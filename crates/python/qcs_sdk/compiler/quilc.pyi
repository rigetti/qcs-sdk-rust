from typing import List, Sequence, Optional, final

from qcs_sdk.qpu.isa import InstructionSetArchitecture
from qcs_sdk.client import QCSClient

DEFAULT_COMPILER_TIMEOUT: float
"""Number of seconds to wait before timing out."""

class QuilcError(RuntimeError):
    """
    A number of errors that can occur when compiling with Quilc:
    - The ``InstructionSetArchitecture`` could not be converted into the Quilc format.
    - Connecting to the Quilc host failed.
    - Quilc returned an error during compilation.
    - The program was not parsed correctly.
    """

    ...


@final
class QuilcClient:
    """
    Client used to communicate with Quilc 
    """
    @staticmethod
    def new_rpcq(endpoint: str) -> QuilcClient:
        """Construct a QuilcClient that uses RPCQ to communicate with Quilc"""
        ...


@final
class CompilerOpts:
    """A set of options that determine the behavior of compiling programs with quilc."""

    def __new__(
        cls,
        /,
        timeout: Optional[float] = DEFAULT_COMPILER_TIMEOUT,
        protoquil: Optional[bool] = None,
    ) -> "CompilerOpts": ...
    @staticmethod
    def default() -> "CompilerOpts": ...

@final
class TargetDevice:
    """
    Architectural description of device to compile for.
    """

    @staticmethod
    def from_isa(
        isa: InstructionSetArchitecture,
    ) -> "TargetDevice":
        """
        Create a ``TargetDevice`` based on an ``InstructionSetArchitecture``.

        :param isa: ``InstructionSetArchitecture`` that describes the target device.

        :raises QuilcError: If the ``InstructionSetArchitecture`` cannot be converted
        into a format that Quilc understands.
        """
        ...
    @staticmethod
    def from_json(
        value: str,
    ) -> "TargetDevice":
        """
        Create a ``TargetDevice`` based on its JSON representation.

        :param value: The JSON representation of a ``TargetDevice`` that describes the target device.

        :raises ValueError: If the JSON is malformed.
        """
        ...

@final
class PauliTerm:
    """Pauli Term used for ConjugatePauliByCliffordRequest."""

    def __new__(
        cls,
        /,
        indices: Sequence[int],
        symbols: Sequence[str],
    ) -> "PauliTerm": ...
    @property
    def indices(self) -> List[int]:
        """Qubit indices onto which the factors of the Pauli Term are applied."""
        ...
    @indices.setter
    def indices(self, value: Sequence[int]): ...
    @property
    def symbols(self) -> List[str]:
        """Ordered factors of the Pauli Term."""
        ...
    @symbols.setter
    def symbols(self, value: Sequence[str]): ...

@final
class ConjugateByCliffordRequest:
    """Request to conjugate a Pauli Term by a Clifford element."""

    def __new__(
        cls,
        /,
        pauli: PauliTerm,
        clifford: str,
    ) -> "ConjugateByCliffordRequest": ...
    @property
    def pauli(self) -> PauliTerm:
        """Pauli Term to conjugate."""
        ...
    @pauli.setter
    def pauli(self, value: PauliTerm): ...
    @property
    def clifford(self) -> str:
        """Clifford element."""
        ...
    @clifford.setter
    def clifford(self, value: str): ...

@final
class ConjugatePauliByCliffordResponse:
    """Pauli Term conjugated by a Clifford element."""

    @property
    def phase(self) -> int:
        """Encoded global phase factor on the emitted Pauli."""
        ...
    @property
    def pauli(self) -> str:
        """Description of the encoded Pauli."""
        ...

@final
class RandomizedBenchmarkingRequest:
    """Request to generate a randomized benchmarking sequence."""

    def __new__(
        cls,
        /,
        depth: int,
        qubits: int,
        gateset: Sequence[str],
        seed: Optional[int] = None,
        interleaver: Optional[str] = None,
    ) -> "RandomizedBenchmarkingRequest": ...
    @property
    def depth(self) -> int:
        """Depth of the benchmarking sequence."""
        ...
    @depth.setter
    def depth(self, value: int): ...
    @property
    def qubits(self) -> int:
        """Number of qubits involved in the benchmarking sequence. Limit 2."""
        ...
    @qubits.setter
    def qubits(self, value: int): ...
    @property
    def gateset(self) -> List[str]:
        """List of Quil programs, each describing a Clifford."""
        ...
    @gateset.setter
    def gateset(self, value: Sequence[str]): ...
    @property
    def seed(self) -> Optional[int]:
        """PRNG seed. Set this to guarantee repeatable results."""
        ...
    @seed.setter
    def seed(self, value: Optional[int]): ...
    @property
    def interleaver(self) -> Optional[str]:
        """Fixed Clifford, specified as a Quil string, to interleave through an RB sequence."""
        ...
    @interleaver.setter
    def interleaver(self, value: Optional[str]): ...

@final
class GenerateRandomizedBenchmarkingSequenceResponse:
    """Randomly generated benchmarking sequence response."""

    @property
    def sequence(self) -> List[List[int]]:
        """Sequence of Cliffords, each expressed as a list of generator indices."""
        ...

def compile_program(
    quil: str,
    target: TargetDevice,
    client: QuilcClient,
    options: Optional[CompilerOpts] = None,
) -> CompilationResult:
    """
    Compile a quil program for a target device.

    :param quil: The Quil program to compile.
    :param target: Architectural description of device to compile for.
    :param client: Client used to send compilation requests to Quilc.
    :param options: Optional compiler options. If ``None``, default values are used.

    :raises QuilcError: If compilation fails.
    """
    ...

async def compile_program_async(
    quil: str,
    target: TargetDevice,
    client: QuilcClient,
    options: Optional[CompilerOpts] = None,
) -> CompilationResult:
    """
    Compile a quil program for a target device.
    (async analog of ``compile_program``)

    :param quil: The Quil program to compile.
    :param target: Architectural description of device to compile for.
    :param client: Client used to send compilation requests to Quilc.
    :param options: Optional compiler options. If ``None``, default values are used.

    :raises QuilcError: If compilation fails.
    """
    ...

@final
class CompilationResult:
    @property
    def program(self) -> str:
        """The compiled program."""
        ...
    @property
    def native_quil_metadata(self) -> Optional[NativeQuilMetadata]:
        """Metadata about the compiled program."""
        ...

@final
class NativeQuilMetadata:
    """
    Metadata about a program compiled to native quil.
    """

    @property
    def final_rewiring(self) -> List[int]:
        """
        Output qubit index relabeling due to SWAP insertion.
        """
        ...
    @property
    def gate_depth(self) -> Optional[int]:
        """Total number of gates in the native Quil program."""
        ...
    @property
    def gate_volume(self) -> Optional[int]:
        """Total number of gates in the native Quil program."""
        ...
    @property
    def multiqubit_gate_depth(self) -> Optional[int]:
        """Maximum number of two-qubit gates in the native Quil program."""
        ...
    @property
    def program_duration(self) -> Optional[float]:
        """Rough estimate of native quil program length in seconds."""
        ...
    @property
    def program_fidelity(self) -> Optional[float]:
        """Rough estimate of the fidelity of the native quil program."""
    @property
    def topological_swaps(self) -> Optional[int]:
        """Total number of swaps in the native Quil program."""
        ...
    @property
    def qpu_runtime_estimation(self) -> Optional[float]:
        """The estimated runtime of the program on a Rigetti QPU, in milliseconds. Available only for protoquil compliant programs."""
        ...


def get_version_info(
    client: QuilcClient,
) -> str:
    """
    Fetch the version information from the running Quilc service.

    :param client: Client used to send compilation requests to Quilc.

    :raises QuilcError: If the is a failure connecting to Quilc.
    """
    ...

async def get_version_info_async(
    client: QuilcClient,
) -> str:
    """
    Fetch the version information from the running Quilc service.
    (async analog of ``get_version_info``)

    :param client: Client used to send compilation requests to Quilc.

    :raises QuilcError: If the is a failure connecting to Quilc.
    """
    ...

def conjugate_pauli_by_clifford(
    request: ConjugateByCliffordRequest,
    client: QuilcClient,
) -> ConjugatePauliByCliffordResponse:
    """
    Given a circuit that consists only of elements of the Clifford group, return its action on a PauliTerm.
    In particular, for Clifford C, and Pauli P, this returns the PauliTerm representing CPC^{\\dagger}.

    :param request: Pauli Term conjugation request.
    :param client: Client used to send compilation requests to Quilc.

    :raises QuilcError: If the is a failure connecting to Quilc or if the request is malformed.
    """
    ...

async def conjugate_pauli_by_clifford_async(
    request: ConjugateByCliffordRequest,
    client: QuilcClient,
) -> ConjugatePauliByCliffordResponse:
    """
    Given a circuit that consists only of elements of the Clifford group, return its action on a PauliTerm.
    In particular, for Clifford C, and Pauli P, this returns the PauliTerm representing CPC^{\\dagger}.
    (async analog of ``conjugate_pauli_by_clifford``)

    :param request: Pauli Term conjugation request.
    :param client: Client used to send compilation requests to Quilc.

    :raises QuilcError: If the is a failure connecting to Quilc or if the request is malformed.
    """
    ...

def generate_randomized_benchmarking_sequence(
    request: RandomizedBenchmarkingRequest,
    client: QuilcClient,
) -> GenerateRandomizedBenchmarkingSequenceResponse:
    """
    Construct a randomized benchmarking experiment on the given qubits, decomposing into
    gateset. If interleaver is not provided, the returned sequence will have the form

        C_1 C_2 ... C_(depth-1) C_inv ,

    where each C is a Clifford element drawn from gateset, C_{< depth} are randomly selected,
    and C_inv is selected so that the entire sequence composes to the identity.  If an
    interleaver G (which must be a Clifford, and which will be decomposed into the native
    gateset) is provided, then the sequence instead takes the form

        C_1 G C_2 G ... C_(depth-1) G C_inv .

    :param request: Pauli Term conjugation request.
    :param client: Client used to send compilation requests to Quilc.

    :raises QuilcError: If the is a failure connecting to Quilc or if the request is malformed.
    """
    ...

async def generate_randomized_benchmarking_sequence_async(
    request: RandomizedBenchmarkingRequest,
    client: QuilcClient,
) -> GenerateRandomizedBenchmarkingSequenceResponse:
    """
    Construct a randomized benchmarking experiment on the given qubits, decomposing into
    gateset. If interleaver is not provided, the returned sequence will have the form

        C_1 C_2 ... C_(depth-1) C_inv ,

    where each C is a Clifford element drawn from gateset, C_{< depth} are randomly selected,
    and C_inv is selected so that the entire sequence composes to the identity.  If an
    interleaver G (which must be a Clifford, and which will be decomposed into the native
    gateset) is provided, then the sequence instead takes the form

        C_1 G C_2 G ... C_(depth-1) G C_inv .
    (async analog of ``generate_randomized_benchmarking_sequence``)

    :param request: Pauli Term conjugation request.
    :param client: Client used to send compilation requests to Quilc.

    :raises QuilcError: If the is a failure connecting to Quilc or if the request is malformed.
    """
