"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""
from enum import Enum

from typing import Dict, List, Optional
from .qpu.quilc import CompilerOpts

class QcsExecutionError(RuntimeError):
    """Error encounteted when executing programs."""
    ...


class Executable:
    """"""
    def __new__(
        registers: Optional[List[str]],
        parameters: Optional[List[ExeParameter]],
        shots: Optional[int],
        compile_with_quilc: Optional[bool],
        compiler_options: Optional[CompilerOpts]
    ) -> "Executable": ...

    async def execute_on_qvm():
        """
        Execute on a QVM which must be available at the configured URL (default http://localhost:5000).

        Raises:
            - ``QcsExecutionError``: If the job fails to execute.
        """
        ...

    async def execute_on_qpu(
        quantum_processor_id: str
    ):
        """
        Compile the program and execute it on a QPU, waiting for results.

        Raises:
            - ``QcsExecutionError``: If the job fails to execute.
        """
        ...
    
    async def retrieve_results(
        job_handle: JobHandle
    ):
        """
        Wait for the results of a job to complete.

        Raises:
            - ``QcsExecutionError``: If there is a problem constructing job results.
        """
        ...


class JobHandle:
    """
    The result of submitting a job to a QPU.
    
    Used to retrieve the results of a job.
    """

    job_id: str
    """Unique ID associated with a single job execution."""

    readout_map: Dict[str, str]
    """
    The readout map from source readout memory locations to the filter pipeline node which publishes the data.
    """


class ExeParameter:
    """
    Program execution parameters.

    Note: The validity of parameters is not checked until execution.
    """

    param_name: str
    """
    References the name of the parameter corresponding to a `DECLARE` statement in the Quil program.
    """

    index: int
    """The index into the memory vector that you're setting."""

    value: float
    """The value to set for the specified memory."""


class Service(Enum):
    Quilc = "Quilc",
    Qvm = "Qvm",
    Qcs = "Qcs",
    Qpu = "Qpu",
