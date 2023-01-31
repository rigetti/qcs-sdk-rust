"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""
from enum import Enum

from typing import Dict, List, Optional
from .qpu.quilc import CompilerOpts
from ._execution_data import QVM, QPU

class QcsExecutionError(RuntimeError):
    """Error encounteted when executing programs."""
    ...


class Executable:
    def __new__(
        cls,
        registers: Optional[List[str]] = None,
        parameters: Optional[List[ExeParameter]] = None,
        shots: Optional[int] = None,
        compile_with_quilc: Optional[bool] = None,
        compiler_options: Optional[CompilerOpts] = None
    ) -> "Executable": ...

    async def execute_on_qvm(self) -> QVM:
        """
        Execute on a QVM which must be available at the configured URL (default http://localhost:5000).

        Raises:
            - ``QcsExecutionError``: If the job fails to execute.
        """
        ...

    async def execute_on_qpu(self, quantum_processor_id: str) -> QPU:
        """
        Compile the program and execute it on a QPU, waiting for results.

        Raises:
            - ``QcsExecutionError``: If the job fails to execute.
        """
        ...
    
    async def retrieve_results(job_handle: JobHandle) -> QPU:
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

    @property
    def job_id(self) -> str:
        """
        Unique ID associated with a single job execution.
        """
        ...

    @property
    def readout_map(self) -> Dict[str, str]:
        """
        The readout map from source readout memory locations to the filter pipeline node which publishes the data.
        """
        ...


class ExeParameter:
    """
    Program execution parameters.

    Note: The validity of parameters is not checked until execution.
    """
    def __new__(
        cls: type["ExeParameter"],
        name: str,
        index: int,
        value: float,
    ) -> "ExeParameter": ...

    @property
    def name(self) -> str: ...
    @name.setter
    def name(self, value: str): ...

    @property
    def index(self) -> int: ...
    @index.setter
    def index(self, value: int): ...

    @property
    def value(self) -> float: ...
    @value.setter
    def value(self, value: float): ...


class Service(Enum):
    Quilc = "Quilc",
    Qvm = "Qvm",
    Qcs = "Qcs",
    Qpu = "Qpu",
