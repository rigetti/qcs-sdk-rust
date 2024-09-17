from collections.abc import Iterable
from typing import Dict, List, Sequence, Mapping, Optional, Union, final

from qcs_sdk.client import QCSClient
from qcs_sdk.qpu import MemoryValues

class SubmissionError(RuntimeError):
    """There was a problem submitting the program to QCS for execution."""

    ...

class QpuApiError(RuntimeError):
    """An error occured while interacting with the QPU API."""

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
    def memory(self) -> Dict[str, MemoryValues]:
        """
        The final state of memory for parameters that were read from and written to during the exectuion of the program.
        """
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

def submit_with_parameter_batch(
    program: str,
    patch_values: Iterable[Mapping[str, Sequence[float]]],
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
) -> List[str]:
    """
    Execute a compiled program on a QPU with multiple sets of `patch_values`.

    This action is *atomic* in that all jobs will be queued, or none of them will. On success, this
    function will return a list of strings where the length and order correspond to the
    `patch_values` given. However, note that execution in the order of given patch values is not
    guaranteed. If there is a failure to queue any of the jobs, then none will be queued.

    Submits an executable `program` to be run on the specified QPU.

    :param program: An executable program (see `translate`).
    :param patch_values: An iterable containing one ore more mapping of symbols to their desired values
        (see `build_patch_values`).
    :param quantum_processor_id: The ID of the quantum processor to run the executable on. This field is required, unless being used with the `ConnectionStrategy.endpoint_id()` execution option.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param execution_options: The ``ExecutionOptions`` to use. If the connection strategy option used is `ConnectionStrategy.endpoint_id("endpoint_id")`, then direct access to "endpoint_id" overrides the `quantum_processor_id` parameter.

    :returns: The ID of the submitted job which can be used to fetch results

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem submitting any of the jobs for execution, or if no
        `patch_values` are given.
    """
    ...

async def submit_with_parameter_batch_async(
    program: str,
    patch_values: Iterable[Mapping[str, Sequence[float]]],
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
) -> List[str]:
    """
    Execute a compiled program on a QPU with multiple sets of `patch_values`.
    (async analog of `submit_with_parameter_batch`)

    This action is *atomic* in that all jobs will be queued, or none of them will. On success, this
    function will return a list of strings where the length and order correspond to the
    `patch_values` given. However, note that execution in the order of given patch values is not
    guaranteed. If there is a failure to queue any of the jobs, then none will be queued.

    Submits an executable `program` to be run on the specified QPU.

    :param program: An executable program (see `translate`).
    :param patch_values: An iterable containing one ore more mapping of symbols to their desired values
        (see `build_patch_values`).
    :param quantum_processor_id: The ID of the quantum processor to run the executable on. This field is required, unless being used with the `ConnectionStrategy.endpoint_id()` execution option.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param execution_options: The ``ExecutionOptions`` to use. If the connection strategy option used is `ConnectionStrategy.endpoint_id("endpoint_id")`, then direct access to "endpoint_id" overrides the `quantum_processor_id` parameter.

    :returns: The ID of the submitted job which can be used to fetch results

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem submitting any of the jobs for execution, or if no
        `patch_values` are given.
    """
    ...

def cancel_job(
    job_id: str,
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
):
    """
    Cancel a job that has yet to begin executing.

    This action is *not* atomic, and will attempt to cancel a job even if it cannot be cancelled. A
    job can be cancelled only if it has not yet started executing.

    Success response indicates only that the request was received. Cancellation is not guaranteed,
    as it is based on job state at the time of cancellation, and is completed on a best effort
    basis.

    :param job_id: The job ID to cancel.
    :param quantum_processor_id: The quantum processor to execute the job on. This parameter is
         required unless using the `ConnectionStrategy.endpoint_id()` execution option.
    :client: - The `QCSClient` to use.
    :execution_options: The `ExecutionOptions` to use. If the connection strategy used is
         ConnectionStrategy.endpoint_id() then direct access to that endpoint overrides the
         `quantum_processor_id` parameter.
    """
    ...

async def cancel_job_async(
    job_id: str,
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
):
    """
    Cancel a job that has yet to begin executing (async analog of `cancel_job`).

    This action is *not* atomic, and will attempt to cancel a job even if it cannot be cancelled. A
    job can be cancelled only if it has not yet started executing.

    Success response indicates only that the request was received. Cancellation is not guaranteed,
    as it is based on job state at the time of cancellation, and is completed on a best effort
    basis.

    :param job_id: The job ID to cancel.
    :param quantum_processor_id: The quantum processor to execute the job on. This parameter is
         required unless using the `ConnectionStrategy.endpoint_id()` execution option.
    :client: - The `QCSClient` to use.
    :execution_options: The `ExecutionOptions` to use. If the connection strategy used is
         ConnectionStrategy.endpoint_id() then direct access to that endpoint overrides the
         `quantum_processor_id` parameter.
    """
    ...

def cancel_jobs(
    job_ids: List[str],
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
):
    """
    Cancel all given jobs that have yet to begin executing.

    This action is *not* atomic, and will attempt to cancel every job even when some jobs cannot be
    cancelled. A job can be cancelled only if it has not yet started executing.

    Success response indicates only that the request was received. Cancellation is not guaranteed,
    as it is based on job state at the time of cancellation, and is completed on a best effort
    basis.

    :param quantum_processor_id: The quantum processor to execute the job on. This parameter is
         required unless using the `ConnectionStrategy.endpoint_id()` execution option.
    :param job_ids: The job IDs to cancel.
    :client: - The `QCSClient` to use.
    :execution_options: The `ExecutionOptions` to use. If the connection strategy used is
         ConnectionStrategy.endpoint_id() then direct access to that endpoint overrides the
         `quantum_processor_id` parameter.
    """
    ...

async def cancel_jobs_async(
    job_ids: List[str],
    quantum_processor_id: Optional[str] = None,
    client: Optional[QCSClient] = None,
    execution_options: Optional[ExecutionOptions] = None,
):
    """
    Cancel all given jobs that have yet to begin executing (async analog of `cancel_jobs`).

    Success response indicates only that the request was received. Cancellation is not guaranteed,
    as it is based on job state at the time of cancellation, and is completed on a best effort
    basis.

    :param quantum_processor_id: The quantum processor to execute the job on. This parameter is
         required unless using the `ConnectionStrategy.endpoint_id()` execution option.
    :param job_ids: The job IDs to cancel.
    :client: - The `QCSClient` to use.
    :execution_options: The `ExecutionOptions` to use. If the connection strategy used is
         ConnectionStrategy.endpoint_id() then direct access to that endpoint overrides the
         `quantum_processor_id` parameter.
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
    :raises QpuApiError: If there was a problem retrieving the results.
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
    :raises QpuApiError: If there was a problem retrieving the results.
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
    @property
    def api_options(self) -> bool:
        """Execution options particular to the API call at the point of execution."""

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
    @property
    def api_options(self) -> bool:
        raise AttributeError("api_options is not readable")
    @api_options.setter
    def api_options(self, api_options: APIExecutionOptions):
        """Execution options particular to the API call at the point of execution."""
    def build(self) -> ExecutionOptions:
        """Build the ``ExecutionOptions`` using the options set in this builder."""

@final
class APIExecutionOptions:
    @staticmethod
    def default() -> APIExecutionOptions:
        """Return APIExecutionOptions with a reasonable set of defaults."""
        ...
    @staticmethod
    def builder() -> APIExecutionOptionsBuilder:
        """Get an ``APIExecutionOptionsBuilder`` that can be used to build a custom set of ``APIExecutionOptions``"""
    @property
    def bypass_settings_protection(self) -> bool:
        """Whether or not to force managed settings to change, if applicable. Subject to additional authorization requirements."""

@final
class APIExecutionOptionsBuilder:
    def __new__(cls) -> APIExecutionOptionsBuilder: ...
    @staticmethod
    def default() -> APIExecutionOptionsBuilder:
        """Return a builder with the default values for ``APIExecutionOptions``"""
        ...
    @property
    def bypass_settings_protection(self) -> bool:
        raise AttributeError("bypass_settings_protection is not readable")
    @bypass_settings_protection.setter
    def bypass_settings_protection(self, bypass_settings_protection: bool):
        """Whether or not to force managed settings to change, if applicable. Subject to additional authorization requirements."""
    def build(self) -> APIExecutionOptions:
        """Build the ``APIExecutionOptions`` using the options set in this builder."""

@final
class ConnectionStrategy:
    """The connection strategy to use when submitting and retrieiving jobs from a quantum processor."""

    @staticmethod
    def default() -> ConnectionStrategy:
        """Get the default connection strategy. Currently, this is ``ConnectionStrategy.gateway()``"""
    @staticmethod
    def gateway() -> ConnectionStrategy:
        """Connect through the publicly accessbile gateway."""
    def is_gateway(self) -> bool:
        """True if the ConnectionStrategy is to connect to the QCS gateway."""
    @staticmethod
    def direct_access() -> ConnectionStrategy:
        """Connect directly to the default endpoint, bypassing the gateway. Should only be used when you have
        direct network access and an active reservation."""
    def is_direct_access(self) -> bool:
        """True if the ConnectionStrategy is to use direct access."""
    @staticmethod
    def endpoint_id(endpoint_id: str) -> ConnectionStrategy:
        """Connect directly to a specific endpoint using its ID."""
    def is_endpoint_id(self) -> bool:
        """True if the ConnectionStrategy is to connect to a particular endpoint ID."""
    def get_endpoint_id(self) -> str:
        """Get the endpoint ID used by the ConnectionStrategy.

        Raises an error if this ConnectionStrategy doesn't use a specific endpoint ID.
        """
