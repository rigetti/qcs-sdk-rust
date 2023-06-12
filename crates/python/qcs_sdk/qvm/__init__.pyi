from typing import final, Mapping, Optional, Sequence, Tuple, Union

from qcs_sdk import RegisterData, QCSClient

from .api import AddressRequest

from . import api as api

@final
class QVMResultData:
    """
    Encapsulates data returned from the QVM after executing a program.
    """

    @staticmethod
    def from_memory_map(memory: Mapping[str, RegisterData]) -> "QVMResultData":
        """
        Build a ``QVMResultData`` from a mapping of register names to a ``RegisterData`` matrix.
        """
        ...
    @property
    def memory(self) -> Mapping[str, RegisterData]:
        """
        Get the mapping of register names (ie. "ro") to a ``RegisterData`` matrix containing the register values.
        """
        ...

@final
class QVMOptions:
    """
    Options avaialable for running programs on the QVM.
    """

    def __new__(cls, timeout_seconds: Optional[float] = None) -> QVMOptions: ...
    @staticmethod
    def default() -> QVMOptions:
        """Get the default set of ``QVMOptions`` used for QVM requests.

        Settings:
            timeout: 30.0 seconds
        """
        ...
    @property
    def timeout(cls):
        """The timeout used for reqeusts to the QVM. If set to none, there is no timeout."""
        ...
    @timeout.setter
    def timeout(cls, timeout: Optional[float]):
        """The timeout used for reqeusts to the QVM. If set to none, there is no timeout."""
        ...

@final
class QVMError(RuntimeError):
    """
    Errors that can occur when running a Quil program on the QVM.
    """

    ...

def run(
    quil: str,
    shots: int,
    addresses: Mapping[str, AddressRequest],
    params: Mapping[str, Union[Sequence[float], Sequence[int]]],
    measurement_noise: Optional[Tuple[float, float, float]] = None,
    gate_noise: Optional[Tuple[float, float, float]] = None,
    rng_seed: Optional[int] = None,
    client: Optional[QCSClient] = None,
    options: Optional[QVMOptions] = None,
) -> QVMResultData:
    """
    Runs the given program on the QVM.

    :param quil: A quil program as a string.
    :param shots: The number of times to run the program. Should be a value greater than zero.
    :param addresses: A mapping of memory region names to an ``AddressRequest`` describing what data to get back for that memory region from the QVM at the end of execution.
    :param params: A mapping of memory region names to their desired values.
    :param client: An optional ``QCSClient`` to use. If unset, creates one using the environemnt configuration (see https://docs.rigetti.com/qcs/references/qcs-client-configuration).
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.

    :returns: A ``QVMResultData`` containing the final state of of memory for the requested readouts after the program finished running.

    :raises QVMError: If one of the parameters is invalid, or if there was a problem communicating with the QVM server.
    """
    ...

async def run_async(
    quil: str,
    shots: int,
    addresses: Mapping[str, AddressRequest],
    params: Mapping[str, Sequence[float]],
    measurement_noise: Optional[Tuple[float, float, float]] = None,
    gate_noise: Optional[Tuple[float, float, float]] = None,
    rng_seed: Optional[int] = None,
    client: Optional[QCSClient] = None,
    options: Optional[QVMOptions] = None,
) -> QVMResultData:
    """
    Asynchronously runs the given program on the QVM.

    :param quil: A quil program as a string.
    :param shots: The number of times to run the program. Should be a value greater than zero.
    :param addresses: A mapping of memory region names to an ``AddressRequest`` describing what data to get back for that memory region from the QVM at the end of execution.
    :param params: A mapping of memory region names to their desired values.
    :param client: An optional ``QCSClient`` to use. If unset, creates one using the environemnt configuration (see https://docs.rigetti.com/qcs/references/qcs-client-configuration).
    :param options: An optional ``QVMOptions`` to use. If unset, uses ``QVMOptions.default()`` for the request.

    :returns: A ``QVMResultData`` containing the final state of of memory for the requested readouts after the program finished running.

    :raises QVMError: If one of the parameters is invalid, or if there was a problem communicating with the QVM server.
    """
    ...
