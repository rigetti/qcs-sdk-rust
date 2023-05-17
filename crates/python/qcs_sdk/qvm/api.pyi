from typing import Dict, List, Optional, final, Union, Tuple
from typing_extensions import Self

from ..qpu.client import QCSClient
from .._register_data import RegisterData

def get_version_info(client: Optional[QCSClient] = None) -> str:
    """
    Gets version information from the running QVM server.

    :returns: The QVM version as a string

    :raises QVMError: If there is a problem communicating with the QVM server.
    """
    ...

async def get_version_info_async(client: Optional[QCSClient] = None) -> str:
    """
    Asynchronously gets version information from the running QVM server.

    :returns: The QVM version as a string

    :raises QVMError: If there is a problem communicating with the QVM server.
    """
    ...

@final
class AddressRequest:
    """
    A description of what values the QVM should return for a memory region.

    Usage:
        ``AddressRequest(True)`` will request all values for a memory region.
        ``AddressRequest(False)`` will request that the memory region be omitted.
        ``AddressRequest(List[int])`` will request that only the values at the given indices be returned.
    """

    def __new__(cls, input: Union[bool, List[int]]): ...
    def inner(self) -> Union[bool, List[int]]: ...
    @staticmethod
    def from_all(all: bool) -> AddressRequest: ...
    @staticmethod
    def from_indices(indices: List[int]) -> AddressRequest: ...
    def as_all(self) -> Optional[bool]: ...
    def as_indices(self) -> Optional[List[int]]: ...
    def to_all(self) -> bool: ...
    def to_indices(self) -> List[int]: ...
    def is_all(self) -> bool: ...
    def is_indices(self) -> bool: ...

@final
class MultishotRequest:
    """The request body needed to make a multishot [`run`] request to the QVM."""

    def __new__(
        cls, quil_instructions: str, trials: int, addresses: Dict[str, AddressRequest]
    ) -> Self: ...
    @property
    def quil_instructions(self) -> str: ...
    @quil_instructions.setter
    def quil_instructions(self, value: str): ...
    @property
    def trials(self) -> int: ...
    @trials.setter
    def trials(self, value: int): ...
    @property
    def addresses(self) -> Dict[str, AddressRequest]: ...
    @addresses.setter
    def addresses(self, value: Dict[str, AddressRequest]): ...

@final
class MultishotResponse:
    """The response body returned by the QVM after a multishot ``run`` request."""

    @property
    def registers(self) -> Dict[str, RegisterData]: ...
    @registers.setter
    def registers(self, value: Dict[str, RegisterData]): ...

def run(
    request: MultishotRequest, client: Optional[QCSClient] = None
) -> MultishotResponse:
    """Executes a program on the QVM"""
    ...

def run_async(
    request: MultishotRequest, client: Optional[QCSClient] = None
) -> MultishotResponse:
    """Executes a program on the QVM"""
    ...

@final
class MultishotMeasureRequest:
    def __new__(
        cls,
        quil_instructions: str,
        trials: int,
        qubits: List[int],
        measurement_noise: Optional[Tuple[float, float, float]],
        rng_seed: Optional[int],
    ) -> Self: ...
    @property
    def quil_instructions(self) -> str: ...
    @quil_instructions.setter
    def quil_instructions(self, value: str): ...
    @property
    def trials(self) -> int: ...
    @trials.setter
    def trials(self, value: int): ...
    @property
    def qubits(self) -> List[int]: ...
    @qubits.setter
    def qubits(self, value: List[int]): ...
    @property
    def measurement_noise(self) -> Optional[Tuple[float, float, float]]: ...
    @measurement_noise.setter
    def measurement_noise(self, value: Optional[Tuple[float, float, float]]): ...
    @property
    def rng_seed(self) -> Optional[int]: ...
    @rng_seed.setter
    def rng_seed(self, value: Optional[int]): ...

@final
class MultishotMeasureResponse:
    """The response body returned by the QVM after a multishot ``run_and_measure`` request."""

    @property
    def results(self) -> Dict[str, List[List[int]]]: ...
    @results.setter
    def results(self, value: Dict[str, List[List[int]]]): ...

def run_and_measure(
    request: MultishotMeasureRequest, client: Optional[QCSClient] = None
) -> MultishotMeasureResponse:
    """Executes a program on the QVM, measuring and returning the state of the qubits at the end of each trial."""
    ...

def run_and_measure_async(
    request: MultishotMeasureRequest, client: Optional[QCSClient] = None
) -> MultishotMeasureResponse:
    """Executes a program on the QVM, measuring and returning the state of the qubits at the end of each trial."""
    ...

# Create type stubs for the measure_expectation function and the types it uses
@final
class ExpectationRequest:
    """The request body needed for a ``measure_expectation`` request to the QVM."""

    def __new__(
        cls, state_preparation: str, operators: List, rng_seed: Optional[int]
    ) -> Self: ...
    @property
    def state_preparation(self) -> str: ...
    @state_preparation.setter
    def state_preparation(self, value: str): ...
    @property
    def operators(self) -> List: ...
    @operators.setter
    def operators(self, value: List): ...
    @property
    def rng_seed(self) -> Optional[int]: ...
    @rng_seed.setter
    def rng_seed(self, value: Optional[int]): ...

@final
class ExpectationResponse:
    """The response body returned by the QVM after a ``measure_expectation`` request."""

    @property
    def expectations(self) -> List[float]: ...
    @expectations.setter
    def expectations(self, expectations: List[float]): ...

def measure_expectation(
    request: ExpectationRequest, client: Optional[QCSClient] = None
) -> ExpectationResponse:
    """Executes a program on the QVM, measuring and returning the expectation value of the given Pauli operators using a prepared state."""
    ...

def measure_expectation_async(
    request: ExpectationRequest, client: Optional[QCSClient] = None
) -> ExpectationResponse:
    """Executes a program on the QVM, measuring and returning the expectation value of the given Pauli operators using a prepared state."""
    ...

@final
class WavefunctionRequest:
    """The request body needed for a ``get_wavefunction`` request to the QVM."""

    def __new__(
        cls,
        compiled_quil: str,
        measurement_noise: Tuple[float, float, float],
        gate_noise: Tuple[float, float, float],
        rng_seed: Optional[int],
    ) -> Self: ...
    @property
    def compiled_quil(self) -> str: ...
    @compiled_quil.setter
    def compiled_quil(self, value: str): ...
    @property
    def measurement_noise(self) -> Tuple[float, float, float]: ...
    @measurement_noise.setter
    def measurement_noise(self, value: Tuple[float, float, float]): ...
    @property
    def gate_noise(self) -> Tuple[float, float, float]: ...
    @gate_noise.setter
    def gate_noise(self, value: Tuple[float, float, float]): ...
    @property
    def rng_seed(self) -> Optional[int]: ...
    @rng_seed.setter
    def rng_seed(self, value: Optional[int]): ...

@final
class WavefunctionResponse:
    """The response body returned by the QVM after a ``get_wavefunction`` request."""

    @property
    def wavefunction(self) -> bytes: ...
    @wavefunction.setter
    def wavefunction(self, value: bytes): ...

def get_wavefunction(
    request: WavefunctionRequest, client: Optional[QCSClient] = None
) -> WavefunctionResponse:
    """Executes a program on the QVM, returning the resulting wavefunction."""
    ...

def get_wavefunction_async(
    request: WavefunctionRequest, client: Optional[QCSClient] = None
) -> WavefunctionResponse:
    """Executes a program on the QVM, returning the resulting wavefunction."""
    ...
