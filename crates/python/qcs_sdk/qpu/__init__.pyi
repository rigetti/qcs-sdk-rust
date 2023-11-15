from typing import Dict, List, Mapping, Sequence, Optional, Union, final, Any

from qcs_sdk.client import QCSClient

from . import (
    api as api,
    isa as isa,
    rewrite_arithmetic as rewrite_arithmetic,
    translation as translation,
)

@final
class ReadoutValues:
    """
    A row of readout values from the QPU. Each row contains all the values emitted
    to a memory reference across all shots. There is a variant for each possible type
    the list of readout values could be.

    Variants:
        - ``integer``: Corresponds to the Quil `BIT`, `OCTET`, or `INTEGER` types.
        - ``real``: Corresponds to the Quil `REAL` type.
        - ``complex``: Corresponds to readout values containing complex numbers

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlying values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """

    def __new__(cls, values: Union[List[int], List[float], List[complex]]):
        """Construct a new ReadoutValues from a list of values."""
        ...
    def inner(self) -> Union[List[int], List[float], List[complex]]:
        """Return the inner list of readout values."""
        ...
    def is_integer(self) -> bool: ...
    def is_real(self) -> bool: ...
    def is_complex(self) -> bool: ...
    def as_integer(self) -> Optional[List[int]]: ...
    def as_real(self) -> Optional[List[float]]: ...
    def as_complex(self) -> Optional[List[complex]]: ...
    def to_integer(self) -> List[int]: ...
    def to_real(self) -> List[float]: ...
    def to_complex(self) -> List[complex]: ...
    @staticmethod
    def from_integer(inner: Sequence[int]) -> "ReadoutValues": ...
    @staticmethod
    def from_real(inner: Sequence[float]) -> "ReadoutValues": ...
    @staticmethod
    def from_complex(inner: Sequence[complex]) -> "ReadoutValues": ...

@final
class QPUResultData:
    """
    Encapsulates data returned from the QPU after executing a job.

    ``QPUResultData`` contains "mappings", which map declared memory regions
    in a program (ie. "ro[0]") to that regions readout key in "readout_values".
    "readout_values" maps those readout keys to the values emitted for that region
    across all shots.
    """

    def __new__(cls, mappings: Mapping[str, str], readout_values: Mapping[str, ReadoutValues]): ...
    @property
    def mappings(self) -> Dict[str, str]:
        """
        Get the mappings of a memory region (ie. "ro[0]") to it's key name in readout_values
        """
        ...
    @property
    def readout_values(self) -> Dict[str, ReadoutValues]:
        """
        Get the mappings of a readout values identifier (ie. "q0") to a set of ``ReadoutValues``
        """
        ...
    def to_raw_readout_data(
        self,
    ) -> RawQPUReadoutData:
        """
        Get a copy of this result data flattened into a ``RawQPUReadoutData``. This reduces
        the contained data down to primitive types, offering a simpler structure at the
        cost of the type safety provided by ``ReadoutValues``.
        """
        ...

@final
class RawQPUReadoutData:
    """
    Encapsulates data returned from the QPU after executing a job. Compared to
    ``QPUReadoutData``, the readout values in this class are returned as lists
    of numbers instead of values wrapped by the ``ReadoutValues`` class.
    """

    @property
    def mappings(self) -> Dict[str, str]:
        """
        Get the mappings of a memory region (ie. "ro[0]") to it's key name in readout_values
        """
        ...
    @property
    def readout_values(self) -> Dict[str, Union[List[int], List[float], List[complex]]]:
        """
        Get the mappings of a readout values identifier (ie. "q0") to a list of those readout values
        """
        ...

class ListQuantumProcessorsError(RuntimeError):
    """A request to list available Quantum Processors failed."""

    ...

def list_quantum_processors(
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> List[str]:
    """
    Returns all available Quantum Processor (QPU) IDs.

    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :raises ListQuantumProcessorsError: If the request to list available QPU IDs failed.
    """
    ...

async def list_quantum_processors_async(
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> List[str]:
    """
    Returns all available Quantum Processor IDs.
    (async analog of ``list_quantum_processors``)

    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :raises ListQuantumProcessorsError: If the request to list available QPU IDs failed.
    """
    ...
