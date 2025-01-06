from __future__ import annotations

from abc import ABC, abstractmethod
from typing import List, TYPE_CHECKING

if TYPE_CHECKING:
    from qcs_sdk.qpu.experimental.randomized_measurements import QubitRandomization


class UnitarySet(ABC):
    """
    An abstract class that defines a set of unitaries for randomized measurements. This interface
    includes both the concrete set of unitaries from which to draw, as well as the Quil
    representation for realizing the unitaries within a quantum program.

    See module level documentation for an example implementation.
    """

    @abstractmethod
    def parameters_per_unitary(self) -> int:
        """
        The number of parameters required to represent the unitary within a set of Quil
        instructions.
        """
        ...

    @abstractmethod
    def unitary_count(self) -> int:
        """The number of unitaries in the set."""
        ...

    @abstractmethod
    def to_parameters(self) -> List[float]:
        """
        Convert the unitary set to a vector of parameters. Each unitary should be represented
        as a contiguous subregion of the list of length `parameters_per_unitary`.

        The length of the list should be equal to `unitary_count` * `parameters_per_unitary`.
        See `ChooseRandomRealSubRegions` for additional detail.
        """
        ...

    @abstractmethod
    def to_instructions(self, qubit_randomizations: List[QubitRandomization]) -> str:
        """
        Given a list of `QubitRandomization`s, return the Quil instructions that realize the unitaries
        randomly drawn for each qubit. For each `QubitRandomization`, the memory region declared by
        `QubitRandomization.destination_declaration` will hold the parameters representing the randomly
        drawn unitary.
        """
        ...
