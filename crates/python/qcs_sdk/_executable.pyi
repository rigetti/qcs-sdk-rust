"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""

from enum import Enum
from typing import Dict, List, Optional, final

from .compiler.quilc import CompilerOpts
from ._execution_data import ExecutionData


class ExecutionError(RuntimeError):
    """Error encountered when executing a program."""
    ...


@final
class Executable:
    """
    The builder interface for executing Quil programs on QVMs and QPUs.
    """

    def __new__(
        cls,
        quil: str,
        /,
        registers: Optional[List[str]] = None,
        parameters: Optional[List[ExeParameter]] = None,
        shots: Optional[int] = None,
        compile_with_quilc: Optional[bool] = None,
        compiler_options: Optional[CompilerOpts] = None,
    ) -> "Executable": ...

    def execute_on_qvm(self) -> ExecutionData:
        """
        Execute on a QVM which must be available at the configured URL (default http://localhost:5000).

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    async def execute_on_qvm_async(self) -> ExecutionData:
        """
        Execute on a QVM which must be available at the configured URL (default http://localhost:5000).
        (async analog of ``Executable.execute_on_qvm``)

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    def execute_on_qpu(self, quantum_processor_id: str, endpoint_id: Optional[str] = None) -> ExecutionData:
        """
        Compile the program and execute it on a QPU, waiting for results.

        :param endpoint_id: execute the compiled program against an explicitly provided endpoint. If `None`,
        the default endpoint for the given quantum_processor_id is used.

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    async def execute_on_qpu_async(self, quantum_processor_id: str, endpoint_id: Optional[str] = None) -> ExecutionData:
        """
        Compile the program and execute it on a QPU, waiting for results.
        (async analog of ``Executable.execute_on_qpu``)

        :param endpoint_id: execute the compiled program against an explicitly provided endpoint. If `None`,
        the default endpoint for the given quantum_processor_id is used.

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    def submit_to_qpu(self, quantum_processor_id: str, endpoint_id: Optional[str] = None) -> JobHandle:
        """
        Compile the program and execute it on a QPU, without waiting for results.

        :param endpoint_id: execute the compiled program against an explicitly provided endpoint. If `None`,
        the default endpoint for the given quantum_processor_id is used.

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    async def submit_to_qpu_async(self, quantum_processor_id: str, endpoint_id: Optional[str] = None) -> JobHandle:
        """
        Compile the program and execute it on a QPU, without waiting for results.
        (async analog of ``Executable.execute_on_qpu``)

        :param endpoint_id: execute the compiled program against an explicitly provided endpoint. If `None`,
        the default endpoint for the given quantum_processor_id is used.

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    def retrieve_results(self, job_handle: JobHandle) -> ExecutionData:
        """
        Wait for the results of a job to complete.

        :raises ExecutionError: If the job fails to execute.
        """
        ...

    async def retrieve_results_async(self, job_handle: JobHandle) -> ExecutionData:
        """
        Wait for the results of a job to complete.
        (async analog of ``Executable.retrieve_results``)

        :raises ExecutionError: If the job fails to execute.
        """
        ...

@final
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

@final
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


@final
class Service(Enum):
    Quilc = "Quilc"
    QVM = "QVM"
    QCS = "QCS"
    QPU = "QPU"
