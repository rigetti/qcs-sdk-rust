from typing import Dict, List, Optional, final

from .client import QCSClient


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

    def is_i32(self) -> bool: ...
    def is_complex32(self) -> bool: ...

    def as_i32(self) -> Optional[List[int]]: ...
    def as_complex32(self) -> Optional[List[complex]]: ...

    def to_i32(self) -> List[int]: ...
    def to_complex32(self) -> List[complex]: ...

    @staticmethod
    def from_i32(inner: List[int]) -> "Register": ...
    @staticmethod
    def from_complex32(inner: List[complex]) -> "Register": ...


@final
class ExecutionResult:
    """Execution readout data from a particular memory location."""

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
    patch_values: Dict[str, List[float]],
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
) -> str:
    """
    Submits an executable `program` to be run on the specified QPU.

    :param program: An executable program (see `translate`).
    :param patch_values: A mapping of symbols to their desired values (see `build_patch_values`).
    :param quantum_processor_id: The ID of the quantum processor to run the executable on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :returns: The ID of the submitted job which can be used to fetch results

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem submitting the program for execution.
    """
    ...


async def submit_async(
    program: str,
    patch_values: Dict[str, List[float]],
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
) -> str:
    """
    Submits an executable `program` to be run on the specified QPU.
    (async analog of ``submit``)

    :param program: An executable program (see `translate`).
    :param patch_values: A mapping of symbols to their desired values (see `build_patch_values`).
    :param quantum_processor_id: The ID of the quantum processor to run the executable on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :returns: The ID of the submitted job which can be used to fetch results

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem submitting the program for execution.
    """
    ...


def retrieve_results(
    job_id: str,
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
) -> ExecutionResults:
    """
    Fetches execution results for the given QCS Job ID.

    :param job_id: The ID of the job to retrieve results for.
    :param quantum_processor_id: The ID of the quantum processor the job ran on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :returns: results from execution.

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem during program execution.
    """
    ...


async def retrieve_results_async(
    job_id: str,
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
) -> ExecutionResults:
    """
    Fetches execution results for the given QCS Job ID.
    (async analog of ``retrieve_results``)

    :param job_id: The ID of the job to retrieve results for.
    :param quantum_processor_id: The ID of the quantum processor the job ran on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :returns: results from execution.

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises SubmissionError: If there was a problem during program execution.
    """
    ...
