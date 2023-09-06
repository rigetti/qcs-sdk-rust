from enum import Enum
from typing import Callable, Dict, List, Sequence, Mapping, Optional, Union, final

from qcs_sdk.client import QCSClient

class SubmissionError(RuntimeError):
    """There was a problem submitting the program to QCS for execution."""

    ...

class RetrieveResultsError(RuntimeError):
    """There was a problem retrieving program execution results from QCS."""

    ...

@final
class Register:
    """
    Data vectors within a single ``ExecutionResult``.

    Variants:
        - ``i32``: A register of 32-bit integers.
        - ``complex64``: A register of 32-bit complex numbers.

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlyting values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """

    def inner(self) -> Union[List[int], List[complex]]:
        """Returns the inner register data."""
        ...
    def is_i32(self) -> bool: ...
    def is_complex32(self) -> bool: ...
    def as_i32(self) -> Optional[List[int]]: ...
    def as_complex32(self) -> Optional[List[complex]]: ...
    def to_i32(self) -> List[int]: ...
    def to_complex32(self) -> List[complex]: ...
    @staticmethod
    def from_i32(inner: Sequence[int]) -> "Register": ...
    @staticmethod
    def from_complex32(inner: Sequence[complex]) -> "Register": ...

@final
class ExecutionResult:
    """Execution readout data from a particular memory location."""

    @staticmethod
    def from_register(register: Register) -> "ExecutionResult":
        """Build an `ExecutionResult` from a `Register`."""
    @property
    def shape(self) -> List[int]:
        """The shape of the result data."""
        ...
    @property
    def data(self) -> Register:
        """The result data for all shots by the particular memory location."""
        ...
    @property
    def dtype(self) -> str:
        """The type of the result data (as a `numpy` `dtype`)."""
        ...

@final
class ExecutionResults:
    """Execution readout data for all memory locations."""

    @property
    def buffers(self) -> Dict[str, ExecutionResult]:
        """
        The readout results of execution, mapping a published filter node to its data.

        See `TranslationResult.ro_sources` which provides the mapping from the filter node name to the name of the memory declaration in the source program.
        """
        ...
    @property
    def execution_duration_microseconds(self) -> Optional[int]:
        """The time spent executing the program."""
        ...

def submit(
    program: str,
    patch_values: Mapping[str, Sequence[float]],
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
) -> str:
    """
    Submits an executable `program` to be run on the specified QPU.

    :param program: An executable program (see `translate`).
    :param patch_values: A mapping of symbols to their desired values (see `build_patch_values`).
    :param quantum_processor_id: The ID of the quantum processor to run the executable on. This field is required, unless being used with the `ConnectionStrategy.endpoint_id()` execution option.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param execution_options: The ``ExecutionOptions`` to use. If the connection strategy option used is `ConnectionStrategy.endpoint_id("endpoint_id")`, then direct access to "endpoint_id" overrides the `quantum_processor_id` parameter.

    :returns: The ID of the submitted job which can be used to fetch results

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem submitting the program for execution.
    """
    ...

async def submit_async(
    program: str,
    patch_values: Mapping[str, Sequence[float]],
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
) -> str:
    """
    Submits an executable `program` to be run on the specified QPU.
    (async analog of ``submit``)

    :param program: An executable program (see `translate`).
    :param patch_values: A mapping of symbols to their desired values (see `build_patch_values`).
    :param quantum_processor_id: The ID of the quantum processor to run the executable on. This field is required, unless being used with the `ConnectionStrategy.endpoint_id()` execution option.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param execution_options: The ``ExecutionOptions`` to use. If the connection strategy option used is `ConnectionStrategy.endpoint_id("endpoint_id")`, then direct access to "endpoint_id" overrides the `quantum_processor_id` parameter.

    :returns: The ID of the submitted job which can be used to fetch results

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem submitting the program for execution.
    """
    ...

def retrieve_results(
    job_id: str,
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
) -> ExecutionResults:
    """
    Fetches execution results for the given QCS Job ID.

    :param job_id: The ID of the job to retrieve results for.
    :param quantum_processor_id: The ID of the quantum processor the job ran on. This field is required, unless being used with the `ConnectionStrategy.endpoint_id()` execution option.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param endpoint_id: retrieve the results of a program submitted to an explicitly provided endpoint. If `None`, the default endpoint for the given quantum_processor_id is used.
    :param execution_options: The ``ExecutionOptions`` to use. If the connection strategy option used is `ConnectionStrategy.endpoint_id("endpoint_id")`, then direct access to "endpoint_id" overrides the `quantum_processor_id` parameter.

    :returns: results from execution.

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem during program execution.
    """
    ...

async def retrieve_results_async(
    job_id: str,
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
) -> ExecutionResults:
    """
    Fetches execution results for the given QCS Job ID.
    (async analog of ``retrieve_results``)

    :param job_id: The ID of the job to retrieve results for.
    :param quantum_processor_id: The ID of the quantum processor the job ran on. This field is required, unless being used with the `ConnectionStrategy.endpoint_id()` execution option.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param endpoint_id: retrieve the results of a program submitted to an explicitly provided endpoint. If `None`, the default endpoint for the given quantum_processor_id is used.
    :param execution_options: The ``ExecutionOptions`` to use. If the connection strategy option used is `ConnectionStrategy.endpoint_id("endpoint_id")`, then direct access to "endpoint_id" overrides the `quantum_processor_id` parameter.

    :returns: results from execution.

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem during program execution.
    """
    ...

@final
class ExecutionOptions:
    @staticmethod
    def default() -> ExecutionOptions:
        """Return ExecutionOptions with a reasonable set of defaults."""
        ...
    @staticmethod
    def builder() -> ExecutionOptionsBuilder:
        """Get an ``ExecutionOptionsBuilder`` that can be used to build a custom set of ``ExecutionOptions``"""
    @property
    def connection_strategy(self) -> ConnectionStrategy:
        """Get the ``ConnectionStrategy`` to be used to connect to the QPU."""
    @property
    def timeout_seconds(self) -> Optional[float]:
        """The time in seconds to wait before timing out a request, if any."""

@final
class ExecutionOptionsBuilder:
    def __new__(cls) -> ExecutionOptionsBuilder: ...
    @staticmethod
    def default() -> ExecutionOptionsBuilder:
        """Return a builder with the default values for ``ExecutionOptions``"""
        ...
    @property
    def connection_strategy(self):
        # This was the least clunky way of expressing connection_strategy as write only.
        # Other methods exposed helper functions that didn't actually exist, while still
        # requiring a getter was defined in some way.
        raise AttributeError("connection_strategy is not readable")
    @connection_strategy.setter
    def connection_strategy(self, connection_strategy: ConnectionStrategy):
        """Set the ``ConnectionStrategy`` used to establish a connection to the QPU."""
    @property
    def timeout_seconds(self):
        raise AttributeError("timeout_seconds is not readable")
    @timeout_seconds.setter
    def timeout_seconds(self, timeout_seconds: Optional[float]):
        """Set the number of seconds to wait before timing out. If set to `None` there is no timeout."""
    def build(self) -> ExecutionOptions:
        """Build the ``ExecutionOptions`` using the options set in this builder."""

@final
class ConnectionStrategy:
    """The connection strategy to use when submitting and retrieiving jobs from a quantum processor."""

    @staticmethod
    def default() -> ConnectionStrategy:
        """Get the default connection strategy. Currently, this is ``ConnectionStrategy.gateway()``"""
    @staticmethod
    def gateway() -> ConnectionStrategy:
        """Connect through the publicly accessbile gateway."""
    @staticmethod
    def direct_access() -> ConnectionStrategy:
        """Connect directly to the default endpoint, bypassing the gateway. Should only be used when you have
        direct network access and an active reservation."""
    @staticmethod
    def endpoint_id(endpoint_id: str) -> ConnectionStrategy:
        """Connect directly to a specific endpoint using its ID."""
