"""
The qcs_sdk module provides an interface to Rigetti Quantum Cloud Services. Allowing users to compile and run Quil programs on Rigetti quantum processors.
"""
from typing import  Dict, List, Optional
from numbers import Number

from .qpu.client import QcsClient

RecalculationTable = List[str]
Memory = Dict[str, List[float]]
PatchValues = Dict[str, List[float]]

class ExecutionError(RuntimeError):
    """Error encountered during program execution submission or when retrieving results."""
    ...


class TranslationError(RuntimeError):
    """Error encountered during program translation."""
    ...


class CompilationError(RuntimeError):
    """Error encountered during program compilation."""
    ...


class RewriteArithmeticError(RuntimeError):
    """Error encountered rewriting arithmetic for program."""
    ...


class DeviceIsaError(ValueError):
    """Error while building Instruction Set Architecture."""
    ...


class RewriteArithmeticResults:
    """
    The result of a call to [`rewrite_arithmetic`] which provides the information necessary to later patch-in memory values to a compiled program.
    """

    @property
    def program(self) -> str:
        """
        The resulting program where gate parameter arithmetic has been replaced with memory references. Before execution, the program memory should be updated using the `recalculation_table`.
        """
        ...
    @program.setter
    def program(self, value: str): ...

    @property
    def recalculation_table(self) -> List[str]:
        """ 
        The recalculation table stores an ordered list of arithmetic expressions, which are to be used when updating the program memory before execution.
        """
        ...
    @recalculation_table.setter
    def recalculation_table(self, value: List[str]): ...


class TranslationResult:
    """
    The result of a call to [`translate`] which provides information about the translated program.
    """

    @property
    def program(self) -> str:
        """
        The compiled program binary.
        """
        ...
    @program.setter
    def program(self, value: str): ...

    @property
    def ro_sources(self) -> Optional[dict]:
        """
        A mapping from the program's memory references to the key used to index the results map.
        """
        ...
    @ro_sources.setter
    def ro_sources(self, value: Optional[dict]): ...


class ExecutionResult:
    """Execution readout data from a particular memory location."""

    @property
    def shape(self) -> List[int]:
        """The shape of the result data."""
        ...
    @shape.setter
    def shape(self, value: List[int]): ...
    
    @property
    def data(self) -> List[Number | List[float]]:
        """The result data. Complex numbers are represented as [real, imaginary]."""
        ...
    @data.setter
    def data(self, value: List[Number | List[float]]): ...

    @property
    def dtype(self) -> str:
        """The type of the result data (as a `numpy` `dtype`)."""
        ...
    @dtype.setter
    def dtype(self, value: str): ...


class ExecutionResults:
    """Execution readout data for all memory locations."""

    @property
    def buffers(self) -> Dict[str, ExecutionResult]:
        """
        The readout results of execution, mapping a published filter node to its data.

        See `TranslationResult.ro_sources` which provides the mapping from the filter node name to the name of the memory declaration in the source program.
        """
        ...
    @buffers.setter
    def buffers(self, value: Dict[str, ExecutionResult]): ...
    
    @property
    def execution_duration_microseconds(self) -> Optional[int]:
        """The time spent executing the program."""
        ...
    @execution_duration_microseconds.setter
    def execution_duration_microseconds(self, value: Optional[int]): ...


class Register:
    """
    Data from an individual register.

    Variants:
        - ``i8``: A register of 8-bit integers.
        - ``i16``: A register of 16-bit integers.
        - ``i32``: A register of 32-bit integers.
        - ``f64``: A register of 64-bit floating point numbers.
        - ``complex64``: A register of 64-bit complex numbers.

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlyting values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """
    
    def is_i8(self) -> bool: ...
    def is_i16(self) -> bool: ...
    def is_i32(self) -> bool: ...
    def is_f64(self) -> bool: ...
    def is_complex64(self) -> bool: ...

    def as_i8(self) -> Optional[List[int]]: ...
    def as_i16(self) -> Optional[List[int]]: ...
    def as_i32(self) -> Optional[List[int]]: ...
    def as_f64(self) -> Optional[List[float]]: ...
    def as_complex64(self) -> Optional[List[complex]]: ...

    def to_i8(self) -> List[int]: ...
    def to_i16(self) -> List[int]: ...
    def to_i32(self) -> List[int]: ...
    def to_f64(self) -> List[float]: ...
    def to_complex64(self) -> List[complex]: ...

    @staticmethod
    def from_i8(inner: List[int]) -> "Register": ...
    @staticmethod
    def from_i16(inner: List[int]) -> "Register": ...
    @staticmethod
    def from_i32(inner: List[int]) -> "Register": ...
    @staticmethod
    def from_f64(inner: List[float]) -> "Register": ...
    @staticmethod
    def from_complex64(inner: List[complex]) -> "Register": ...


async def compile(
    quil: str,
    target_device: str,
    client: Optional[QcsClient] = None,
    *,
    timeout: int = 30,
) -> str:
    """
    Uses quilc to convert a quil program to native Quil.

    Args:
        quil: A Quil program.
        target_device: A JSON encoded description of the Quantum Abstract Machine Architecture.
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    Keyword Args:
        timeout: The number of seconds to wait before timing out. If set to None, there is no timeout (default: 30).

    Returns:
        An Awaitable that resolves to the native Quil program.

    Raises:
        - ``LoadError`` If there is an issue loading the QCS Client configuration.
        - ``DeviceIsaError`` If the `target_device` is misconfigured.
        - ``CompilationError`` If the program could not compile.
    """
    ...


def rewrite_arithmetic(
    native_quil: str,
) -> RewriteArithmeticResults:
    """
    Rewrites parametric arithmetic such that all gate parameters are only memory references
    to a newly declared memory location (__SUBST).

    Args:
        native_quil: A Quil program.

    Returns:
        A dictionary with the rewritten program and recalculation table (see `RewriteArithmeticResults`).
    
    Raises:
        - ``TranslationError`` If the program could not be translated.
        - ``RewriteArithmeticError`` If the program arithmetic cannot be evaluated.
    """
    ...


def build_patch_values(
    recalculation_table: RecalculationTable,
    memory: Memory,
) -> PatchValues:
    """
    Evaluate the expressions in recalculation_table using the numeric values
    provided in memory.

    Args:
        recalculation_table: an ordered list of arithmetic expressions, which are to be used when updating the program memory before execution (see `rewrite_arithmetic`).
        memory: A mapping of symbols to their values (e.g. `{"theta": [0.0]}`).

    Returns:
        A dictionary that maps each symbol to the value it should be patched with.

    Raises:
        - ``TranslationError`` If the expressions in `recalculation_table` could not be evaluated.
    """
    ...


async def translate(
    native_quil: str,
    num_shots: int,
    quantum_processor_id: str,
    client: Optional[QcsClient] = None,
) -> TranslationResult:
    """
    Translates a native Quil program into an executable.

    Args:
        native_quil: A Quil program.
        num_shots: The number of shots to perform.
        quantum_processor_id: The ID of the quantum processor the executable will run on (e.g. "Aspen-M-2").
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    Returns:
        An Awaitable that resolves to a dictionary with the compiled program, memory descriptors, and readout sources (see `TranslationResult`).
    
    Raises:
        - ``LoadError`` If there is an issue loading the QCS Client configuration.
        - ``TranslationError`` If the `native_quil` program could not be translated.
    """
    ...


async def submit(
    program: str,
    patch_values: Dict[str, List[float]],
    quantum_processor_id: str,
    client: Optional[QcsClient] = None,
) -> str:
    """
    Submits an executable `program` to be run on the specified QPU.

    Args:
        program: An executable program (see `translate`).
        patch_values: A mapping of symbols to their desired values (see `build_patch_values`).
        quantum_processor_id: The ID of the quantum processor to run the executable on.
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    Returns:
        An Awaitable that resolves to the ID of the submitted job.
    
    Raises:
        - ``LoadError`` If there is an issue loading the QCS Client configuration.
        - ``ExecutionError`` If there was a problem during program execution.
    """
    ...


async def retrieve_results(
    job_id: str,
    quantum_processor_id: str,
    client: Optional[QcsClient] = None,
) -> ExecutionResults:
    """
    Fetches results for the corresponding job ID.

    Args:
        job_id: The ID of the job to retrieve results for.
        quantum_processor_id: The ID of the quanutum processor the job ran on.
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    Returns:
        An Awaitable that resolves to a dictionary describing the results of the execution and its duration (see `ExecutionResults`).

    Raises:
        - ``LoadError`` If there is an issue loading the QCS Client configuration.
        - ``ExecutionError`` If there was a problem fetching execution results.
    """
    ...


async def get_quilc_version(
    client: Optional[QcsClient] = None,
) -> str:
    """
    Returns the version number of the running quilc server.

    Args:
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    
    Raises:
        - ``LoadError`` If there is an issue loading the QCS Client configuration.
        - ``CompilationError`` If there is an issue fetching the version from the quilc compiler.
    """
    ...


async def get_quilc_version(
    client: Optional[QcsClient] = None,
) -> str:
    """
    Returns the version number of the running quilc server.

    Args:
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    
    Raises:
        - ``LoadError`` If there is an issue loading the QCS Client configuration.
        - ``CompilationError`` If there is an issue fetching the version from the quilc compiler.
    """
    ...

async def list_quantum_processors(
    client: Optional[QcsClient] = None,
    timeout: Optional[float] = None,
) -> List[str]:
    """
    Returns all available Quantum Processor IDs.

    Args:
        client: The QcsClient to use. Loads one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
        timeout: Maximum duration to wait for API calls to complete, in seconds.
    """
    ...