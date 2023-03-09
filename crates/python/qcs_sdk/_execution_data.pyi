"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""

import datetime
from typing import Optional, final

import numpy as np
from numpy.typing import NDArray

from .qpu import QPUResultData
from .qvm import QVMResultData

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
