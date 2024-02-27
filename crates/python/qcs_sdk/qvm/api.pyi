from typing import Dict, List, Sequence, Mapping, Optional, Tuple, final
from typing_extensions import Self

from qcs_sdk import RegisterData
from qcs_sdk.client import QCSClient
from qcs_sdk.qvm import QVMOptions, QVMClient

def get_version_info(
    client: QVMClient, options: Optional[QVMOptions] = None
) -> str:
    """
    Gets version information from the running QVM server.

    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.

    :returns: The QVM version as a string

    :raises QVMError: If there is a problem communicating with the QVM server.
    """
    ...

async def get_version_info_async(
    client: QVMClient, options: Optional[QVMOptions] = None
) -> str:
    """
    Asynchronously gets version information from the running QVM server.

    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.

    :returns: The QVM version as a string

    :raises QVMError: If there is a problem communicating with the QVM server.
    """
    ...

@final
class AddressRequest:
    """
    A description of what values the QVM should return for a memory region.
    """

    @staticmethod
    def include_all() -> AddressRequest:
        """
        Request all values for a memory region.
        """
        ...
    @staticmethod
    def exclude_all() -> AddressRequest:
        """
        Exclude all values for a memory region.
        """
        ...
    @staticmethod
    def from_indices(indices: Sequence[int]) -> AddressRequest:
        """
        Request values at the given indices in a memory region.
        """
        ...

@final
class MultishotRequest:
    """The request body needed to make a multishot [`run`] request to the QVM."""

    def __new__(
        cls,
        compiled_quil: str,
        trials: int,
        addresses: Mapping[str, AddressRequest],
        measurement_noise: Optional[Tuple[float, float, float]],
        gate_noise: Optional[Tuple[float, float, float]],
        rng_seed: Optional[int],
    ) -> Self: ...
    @property
    def compiled_quil(self) -> str: ...
    @compiled_quil.setter
    def compiled_quil(self, value: str): ...
    @property
    def trials(self) -> int: ...
    @trials.setter
    def trials(self, value: int): ...
    @property
    def addresses(self) -> Dict[str, AddressRequest]: ...
    @addresses.setter
    def addresses(self, value: Dict[str, AddressRequest]): ...
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
class MultishotResponse:
    """The response body returned by the QVM after a multishot ``run`` request."""

    @property
    def registers(self) -> Dict[str, RegisterData]: ...
    @registers.setter
    def registers(self, value: Mapping[str, RegisterData]): ...

def run(
    request: MultishotRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> MultishotResponse:
    """Executes a program on the QVM

    :param request: The ``MultishotRequest`` to use.
    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.

    """
    ...

def run_async(
    request: MultishotRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> MultishotResponse:
    """Executes a program on the QVM

    :param request: The ``MultishotRequest`` to use.
    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.
    """
    ...

@final
class MultishotMeasureRequest:
    def __new__(
        cls,
        compiled_quil: str,
        trials: int,
        qubits: Sequence[int],
        measurement_noise: Optional[Tuple[float, float, float]] = None,
        gate_noise: Optional[Tuple[float, float, float]] = None,
        rng_seed: Optional[int] = None,
    ) -> Self: ...
    @property
    def compiled_quil(self) -> str: ...
    @compiled_quil.setter
    def compiled_quil(self, value: str): ...
    @property
    def trials(self) -> int: ...
    @trials.setter
    def trials(self, value: int): ...
    @property
    def qubits(self) -> List[int]: ...
    @qubits.setter
    def qubits(self, value: Sequence[int]): ...
    @property
    def measurement_noise(self) -> Optional[Tuple[float, float, float]]: ...
    @measurement_noise.setter
    def measurement_noise(self, value: Optional[Tuple[float, float, float]]): ...
    @property
    def gate_noise(self) -> Tuple[float, float, float]: ...
    @gate_noise.setter
    def gate_noise(self, value: Tuple[float, float, float]): ...
    @property
    def rng_seed(self) -> Optional[int]: ...
    @rng_seed.setter
    def rng_seed(self, value: Optional[int]): ...

def run_and_measure(
    request: MultishotMeasureRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> List[List[int]]:
    """Executes a program on the QVM, measuring and returning the state of the qubits at the end of each trial."""
    ...

def run_and_measure_async(
    request: MultishotMeasureRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> List[List[int]]:
    """Executes a program on the QVM, measuring and returning the state of the qubits at the end of each trial."""
    ...

# Create type stubs for the measure_expectation function and the types it uses
@final
class ExpectationRequest:
    """The request body needed for a ``measure_expectation`` request to the QVM."""

    def __new__(
        cls, state_preparation: str, operators: Sequence, rng_seed: Optional[int] = None
    ) -> Self: ...
    @property
    def state_preparation(self) -> str: ...
    @state_preparation.setter
    def state_preparation(self, value: str): ...
    @property
    def operators(self) -> List: ...
    @operators.setter
    def operators(self, value: Sequence): ...
    @property
    def rng_seed(self) -> Optional[int]: ...
    @rng_seed.setter
    def rng_seed(self, value: Optional[int]): ...

def measure_expectation(
    request: ExpectationRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> List[float]:
    """
    Executes a program on the QVM, measuring and returning the expectation value of the given Pauli operators using a prepared state.

    :param request: The ``ExpectationRequest`` to use.
    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.
    """
    ...

def measure_expectation_async(
    request: ExpectationRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> List[float]:
    """
    Executes a program on the QVM, measuring and returning the expectation value of the given Pauli operators using a prepared state.

    :param request: The ``ExpectationRequest`` to use.
    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.
    """
    ...

@final
class WavefunctionRequest:
    """The request body needed for a ``get_wavefunction`` request to the QVM."""

    def __new__(
        cls,
        compiled_quil: str,
        measurement_noise: Optional[Tuple[float, float, float]] = None,
        gate_noise: Optional[Tuple[float, float, float]] = None,
        rng_seed: Optional[int] = None,
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

def get_wavefunction(
    request: WavefunctionRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> bytes:
    """
    Executes a program on the QVM, returning the resulting wavefunction.

    :param request: The ``WavefunctionRequest`` to use.
    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.
    """
    ...

def get_wavefunction_async(
    request: WavefunctionRequest,
    client: QVMClient,
    options: Optional[QVMOptions] = None,
) -> List[int]:
    """
    Executes a program on the QVM, returning the resulting wavefunction.

    :param request: The ``WavefunctionRequest`` to use.
    :param client: Client used to send requests to QVM.
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.
    """
    ...
