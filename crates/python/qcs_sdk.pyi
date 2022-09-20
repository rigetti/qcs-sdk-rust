from typing import Any, Awaitable, Dict, List, Optional, TypedDict
from numbers import Number

RecalculationTable = List[str]
Memory = Dict[str, List[float]]
PatchValues = Dict[str, List[float]]

class RewriteArithmeticResults(TypedDict):
    program: str
    recalculation_table: RecalculationTable

class TranslationResult(TypedDict):
    memory_descriptors: Optional[Dict[str, Any]]
    program: str
    ro_sources: Optional[List[List[str]]]
    settings_timestamp: Optional[str]

class ExecutionResult(TypedDict):
    shape: List[int]
    data: List[Number]
    dtype: str

class ExecutionResults(TypedDict):
    buffers: Dict[str, ExecutionResult]
    execution_duration_microseconds: Optional[int]

def compile(quil: str, target_device: str) -> Awaitable[str]:
    """
    Uses quilc to convert a quil program to native Quil.

    Args:
        quil (str): A quil program.
        target_device: The device to target for nativization (e.g. "Aspen-M-2")

    Returns:
        An Awaitable that resolves to a string containing a quil program native to the target device.
    """
    ...

def rewrite_arithmetic(native_quil: str) -> Awaitable[RewriteArithmeticResults]:
    """
    Rewrites parametric arithmetic such that all gate parameters are only memory references
    to newly declared memory location (`__SUBST`).

    Args:
        native_quil: A quil program.

    Returns:
        An Awaitable that resolves to a dictionary containing two keys:
            - "program": the rewritten program.
            - "recalculation_table": A list of expressions used to populate memory (see build_patch_values).
    """
    ...

def build_patch_values(
    recalculation_table: RecalculationTable, memory: Memory
) -> Awaitable[PatchValues]:
    """
    Evaluate the expressions in recalculation_table using the numeric values
    provided in memory.

    Args:
        recalculation_table: A table of expressions to evaluate and use to populate memory (see rewrite_arithmetic).
        memory: A mapping of symbols to their values (e.g. {"theta": [0.0]}).

    Returns:
        An Awaitable that resolves to a mapping of each symbol to the value it should be patched with.
    """
    ...

def translate(
    native_quil: str, num_shots: int, quantum_processor_id: str
) -> Awaitable[TranslationResult]:
    """
    Translates a native Quil program into an executable.

    Args:
        native_quil: A quil program.
        num_shots: The number of shots to perform.
        quantum_processor_id: The ID of the quantum processor the executable will run on (e.g. "Aspen-M-2").

    Returns:
        An awaitable that resolves to a dictionary containing four keys:
            - "memory_descriptors": The memory defined in the program.
            - "program": The translated program.
            - "ro_sources": The memory locations used for readout.
            - "settings_timestamp": ISO8601 timestamp of the settings used to translate the program.
                Translation is deterministic; a program translated twice with the same settings by the
                same version of the service will have identical output.
    """
    ...

def submit(
    program: str, patch_values: Dict[str, List[float]], quantum_processor_id: str
) -> Awaitable[str]:
    """
    Submits an executable `program` to be run on the specified QPU.

    Args:
        program: An executable program (see translate).
        patch_values: A mapping of symbols to their desired values (see build_patch_values).
        quantum_processor_id: The ID of the quantum processor to run the executable on.

    Returns:
        An Awaitable that resolves to the ID of the submitted job.
    """
    ...

def retrieve_results(
    job_id: str, quantum_processor_id: str
) -> Awaitable[ExecutionResults]:
    """
    Fetches results for the corresponding job ID.

    Args:
        job_id: The ID of the job to retrieve results for.
        quantum_processor_id: The ID of the quanutum processor the job ran on.

    Returns:
        An Awaitable that resolves to a dictionary describing the execution results:
            - "buffers": A dictionary mapping memory buffers to their readout values.
            - "execution_duration_microseconds": The duration in microseconds the job took to execute.
    """
    ...
