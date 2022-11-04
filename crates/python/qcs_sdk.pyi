"""
The qcs_sdk module provides an interface to Rigetti Quantum Cloud Services. Allowing users to compile and run Quil programs on Rigetti quantum processors.
"""
from typing import Any, Dict, List, Optional, TypedDict
from numbers import Number

RecalculationTable = List[str]
Memory = Dict[str, List[float]]
PatchValues = Dict[str, List[float]]

class RewriteArithmeticResults(TypedDict):
    program: str
    """
    The resulting program where gate parameter arithmetic has been replaced with memory references. Before execution, the program memory should be updated using the `recalculation_table`.
    """

    recalculation_table: RecalculationTable
    """ 
    The recalculation table stores an ordered list of arithmetic expressions, which are to be used when updating the program memory before execution.
    """

class TranslationResult(TypedDict):
    memory_descriptors: Optional[Dict[str, Any]]
    """
    A map from the name of memory (declared with `DECLARE`) to the size and type of that memory.
    """

    program: str
    """
    The compiled program binary.
    """

    ro_sources: Optional[List[List[str]]]
    """
    A mapping from the program's memory references to the key used to index the results map.
    """

    settings_timestamp: Optional[str]
    """
    The timestamp of the settings used during translation.
    """

class ExecutionResult(TypedDict):
    shape: List[int]
    """
    The shape of the result data.
    """

    data: List[Number | List[float]]
    """
    The result data. Complex numbers are represented as [real, imaginary].
    """

    dtype: str
    """
    The type of the result data (as a `numpy` `dtype`).
    """

class ExecutionResults(TypedDict):
    buffers: Dict[str, ExecutionResult]
    """
    The readout results of execution, mapping a published filter node to its data.

    See `TranslationResult.ro_sources` which provides the mapping from the filter node name to the name of the memory declaration in the source program.
    """

    execution_duration_microseconds: Optional[int]
    """
    The time spent executing the program.
    """

async def compile(quil: str, target_device: str, *, timeout: int = 30) -> str:
    """
    Uses quilc to convert a quil program to native Quil.

    Args:
        quil: A Quil program.
        target_device: A JSON encoded description of the Quantum Abstract Machine Architecture.

    Keyword Args:
        timeout: The number of seconds to wait before timing out. If set to None, there is no timeout (default: 30).

    Returns:
        An Awaitable that resolves to the native Quil program.
    """
    ...

def rewrite_arithmetic(native_quil: str) -> RewriteArithmeticResults:
    """
    Rewrites parametric arithmetic such that all gate parameters are only memory references
    to a newly declared memory location (__SUBST).

    Args:
        native_quil: A Quil program.

    Returns:
        A dictionary with the rewritten program and recalculation table (see `RewriteArithmeticResults`).
    """
    ...

def build_patch_values(
    recalculation_table: RecalculationTable, memory: Memory
) -> PatchValues:
    """
    Evaluate the expressions in recalculation_table using the numeric values
    provided in memory.

    Args:
        recalculation_table: an ordered list of arithmetic expressions, which are to be used when updating the program memory before execution (see `rewrite_arithmetic`).
        memory: A mapping of symbols to their values (e.g. `{"theta": [0.0]}`).

    Returns:
        A dictionary that maps each symbol to the value it should be patched with.
    """
    ...

async def translate(
    native_quil: str, num_shots: int, quantum_processor_id: str
) -> TranslationResult:
    """
    Translates a native Quil program into an executable.

    Args:
        native_quil: A Quil program.
        num_shots: The number of shots to perform.
        quantum_processor_id: The ID of the quantum processor the executable will run on (e.g. "Aspen-M-2").

    Returns:
        An Awaitable that resolves to a dictionary with the compiled program, memory descriptors, and readout sources (see `TranslationResult`).
    """
    ...

async def submit(
    program: str, patch_values: Dict[str, List[float]], quantum_processor_id: str
) -> str:
    """
    Submits an executable `program` to be run on the specified QPU.

    Args:
        program: An executable program (see `translate`).
        patch_values: A mapping of symbols to their desired values (see `build_patch_values`).
        quantum_processor_id: The ID of the quantum processor to run the executable on.

    Returns:
        An Awaitable that resolves to the ID of the submitted job.
    """
    ...

async def retrieve_results(job_id: str, quantum_processor_id: str) -> ExecutionResults:
    """
    Fetches results for the corresponding job ID.

    Args:
        job_id: The ID of the job to retrieve results for.
        quantum_processor_id: The ID of the quanutum processor the job ran on.

    Returns:
        An Awaitable that resolves to a dictionary describing the results of the execution and its duration (see `ExecutionResults`).
    """
    ...

async def get_quilc_version() -> str:
    """
    Returns the version number of the running quilc server.
    """
    ...
