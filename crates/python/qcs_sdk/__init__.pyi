import datetime
from enum import Enum
from typing import Dict, List, Sequence, Optional, Union, final

import numpy as np
from numpy.typing import NDArray

from qcs_sdk.qpu import QPUResultData
from qcs_sdk.qvm import QVMResultData
from qcs_sdk.compiler.quilc import CompilerOpts

from qcs_sdk.client import QCSClient as QCSClient

from . import qpu as qpu
from . import qvm as qvm
from . import compiler as compiler
from . import client as client

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
        registers: Optional[Sequence[str]] = None,
        parameters: Optional[Sequence[ExeParameter]] = None,
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
        cls,
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

class RegisterMatrixConversionError(ValueError):
    """Error that may occur when building a ``RegisterMatrix`` from execution data."""

    ...

@final
class RegisterMatrix:
    """
    Values in a 2-dimensional ``ndarray`` representing the final shot value in each memory reference across all shots.
    Each variant corresponds to the possible data types a register can contain.

    Variants:
        ``integer``: Corresponds to the Quil `BIT`, `OCTET`, or `INTEGER` types.
        ``real``: Corresponds to the Quil `REAL` type.
        ``complex``: Registers containing complex numbers.

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlying values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """

    def is_integer(self) -> bool: ...
    def is_real(self) -> bool: ...
    def is_complex(self) -> bool: ...
    def as_integer(self) -> Optional[NDArray[np.int64]]: ...
    def as_real(self) -> Optional[NDArray[np.float64]]: ...
    # In numpy `complex128` is a complex number made up of two `f64`s.
    def as_complex(self) -> Optional[NDArray[np.complex128]]: ...
    def to_integer(self) -> NDArray[np.int64]: ...
    def to_real(self) -> NDArray[np.float64]: ...
    def to_complex(self) -> NDArray[np.complex128]: ...
    @staticmethod
    def from_integer(inner: NDArray[np.int64]) -> "RegisterMatrix": ...
    @staticmethod
    def from_real(inner: NDArray[np.float64]) -> "RegisterMatrix": ...
    @staticmethod
    def from_complex(inner: NDArray[np.complex128]) -> "RegisterMatrix": ...

@final
class RegisterMap:
    """A map of register names (ie. "ro") to a ``RegisterMatrix`` containing the values of the register."""

    def get_register_matrix(self, register_name: str) -> Optional[RegisterMatrix]:
        """Get the ``RegisterMatrix`` for the given register. Returns `None` if the register doesn't exist."""
        ...

@final
class ResultData:
    """
    Represents the two possible types of data returned from either the QVM or a real QPU.
    Each variant contains the original data returned from its respective executor.

    Usage
    -----

    Your usage of ``ResultData`` will depend on the types of programs you are running and where.
    The `to_register_map()` method will attempt to build ``RegisterMap`` out of the data, where each
    register name is mapped to a 2-dimensional rectangular ``RegisterMatrix`` where each row
    represents the final values in each register index for a particular shot. This is often the
    desired form of the data and it is _probably_ what you want. This transformation isn't always
    possible, in which case `to_register_map()` will return an error.

    To understand why this transformation can fail, we need to understand a bit about how readout data is
    returned from the QVM and from a real QPU:

    The QVM treats each `DECLARE` statement as initialzing some amount of memory. This memory works
    as one might expect it to. It is zero-initalized, and subsequent writes to the same region
    overwrite the previous value. The QVM returns memory at the end of every shot. This means
    we get the last value in every memory reference for each shot, which is exactly the
    representation we want for a ``RegisterMatrix``. For this reason, `to_register_map()` should
    always succeed for ``ResultData::Qvm``.

    The QPU on the other hand doesn't use the same memory model as the QVM. Each memory reference
    (ie. "ro[0]") is more like a stream than a value in memory. Every `MEASURE` to a memory
    reference emits a new value to said stream. This means that the number of values per memory
    reference can vary per shot. For this reason, it's not always clear what the final value in
    each shot was for a particular reference. When this is the case, `to_register_map()` will return
    an error as it's impossible to build a correct ``RegisterMatrix``  from the data without
    knowing the intent of the program that was run. Instead, it's recommended to build the
    ``RegisterMatrix`` you need from the inner ``QPUResultData`` data using the knowledge of your
    program to choose the correct readout values for each shot.

    Variants:
        - ``qvm``: Data returned from the QVM, stored as ``QVMResultData``
        - ``qpu``: Data returned from the QPU, stored as ``QPUResultData``

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlying values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.
    """

    def to_register_map(self) -> RegisterMap:
        """
        Convert ``ResultData`` from its inner representation as ``QVMResultData`` or
        ``QPUResultData`` into a ``RegisterMap``. The ``RegisterMatrix`` for each register will be
        constructed such that each row contains all the final values in the register for a single shot.

        Errors
        ------

        Raises a ``RegisterMatrixConversionError`` if the inner execution data for any of the
        registers would result in a jagged matrix. ``QPUResultData`` data is captured per measure,
        meaning a value is returned for every measure to a memory reference, not just once per shot.
        This is often the case in programs that use mid-circuit measurement or dynamic control flow,
        where measurements to the same memory reference might occur multiple times in a shot, or be
        skipped conditionally. In these cases, building a rectangular ``RegisterMatrix`` would
        necessitate making assumptions about the data that could skew the data in undesirable ways.
        Instead, it's recommended to manually build a matrix from ``QPUResultData`` that accurately
        selects the last value per-shot based on the program that was run.
        """
        ...
    def inner(
        self,
    ) -> Union[QVMResultData, QPUResultData]:
        """Returns the inner result data"""
        ...
    def is_qvm(self) -> bool: ...
    def is_qpu(self) -> bool: ...
    def as_qvm(self) -> Optional[QVMResultData]: ...
    def as_qpu(self) -> Optional[QPUResultData]: ...
    def to_qvm(self) -> QVMResultData: ...
    def to_qpu(self) -> QPUResultData: ...
    @staticmethod
    def from_qvm(inner: QVMResultData) -> "ResultData": ...
    @staticmethod
    def from_qpu(inner: QPUResultData) -> "ResultData": ...

@final
class ExecutionData:
    @property
    def result_data(self) -> ResultData: ...
    @result_data.setter
    def result_data(self, result_data: ResultData): ...
    @property
    def duration(self) -> Optional[datetime.timedelta]: ...
    @duration.setter
    def duration(self, duration: Optional[datetime.timedelta]): ...

@final
class RegisterData:
    """
    Values present in a register that are one of a set of variants.

    Variants:
        - ``i8``: Corresponds to the Quil `BIT` or `OCTET` types.
        - ``i16``: Corresponds to the Quil `INTEGER` type.
        - ``f64``: Corresponds to the Quil `REAL` type.
        - ``complex32``: Results containing complex numbers.

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlying values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """

    def inner(
        self,
    ) -> Union[List[List[int]], List[List[float]], List[List[complex]]]:
        """Returns the inner value."""
        ...
    def is_i8(self) -> bool: ...
    def is_i16(self) -> bool: ...
    def is_f64(self) -> bool: ...
    def is_complex32(self) -> bool: ...
    def as_i8(self) -> Optional[List[List[int]]]: ...
    def as_i16(self) -> Optional[List[List[int]]]: ...
    def as_f64(self) -> Optional[List[List[float]]]: ...
    def as_complex32(self) -> Optional[List[List[complex]]]: ...
    def to_i8(self) -> List[List[int]]: ...
    def to_i16(self) -> List[List[int]]: ...
    def to_f64(self) -> List[List[float]]: ...
    def to_complex32(self) -> List[List[complex]]: ...
    @staticmethod
    def from_i8(inner: Sequence[Sequence[int]]) -> "RegisterData": ...
    @staticmethod
    def from_i16(inner: Sequence[Sequence[int]]) -> "RegisterData": ...
    @staticmethod
    def from_f64(inner: Sequence[Sequence[float]]) -> "RegisterData": ...
    @staticmethod
    def from_complex32(inner: Sequence[Sequence[complex]]) -> "RegisterData": ...

def reset_logging():
    """
    Reset all caches for logging configuration within this library, allowing the most recent Python-side
    changes to be applied.

    See <https://docs.rs/pyo3-log/latest/pyo3_log/> for more information.
    """
