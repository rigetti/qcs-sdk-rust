from typing import Dict, List, Optional, Self, Tuple, final

import numpy as np
from numpy.typing import NDArray


@final
class RandomError(ValueError):
    """
    An error that may occur while initializing a seed value or
    generating a QPU PRNG sequence.
    """
    ...

@final
class PrngSeedValue:
    """A seed value for the Rigetti QPU PRNG."""

    def __new__(cls, value: int) -> Self: ... 


@final
class ChooseRandomRealSubRegions:
    """
    A class that represents a QPU extern call to pseudo-randomly choose sub-regions of
    a source memory declaration. The parameters of the call function are as follows:

    * `destination` - The destination memory region to copy the pseudo-randomly chosen
        sub-region to.
    * `source` - The source memory region to choose sub-regions from.
    * `sub_region_size` - The size of each sub-region to choose.
    * `seed` - A memory reference to an integer value that will be used to seed the PRNG.
        This value will be mutated to the next element in the PRNG sequence, so you may
        use it to generate subsequent pseudo-random values.

    Note, `len(destination) % sub_region-size` and `len(source) % sub_region_size` must be 0.

    # Example

    The following example declares a source with 12 real values, representing 4 sub-regions
    each of size 3. The destination memory region is declared with 3 real values, representing
    a single sub-region. The seed memory region is declared with a single integer value.

    The `Call` instruction will pseudo-randomly choose a sub-region of the source memory region
    and copy it to the destination memory region according to the seed value. The seed value is
    mutated to the next element in the PRNG sequence.

    >>> program = Program()
    >>> destination = Declaration("destination", Vector(ScalarType.Real, 3), None)
    >>> program.add_instruction(Instruction.from_declaration(destination))
    >>> source = Declaration("source", Vector(ScalarType.Real, 12), None)
    >>> program.add_instruction(Instruction.from_declaration(source))
    >>> seed = Declaration("seed", Vector(ScalarType.Integer, 1), None)
    >>> program.add_instruction(Instruction.from_declaration(seed))
    >>> pragma_extern = Pragma("EXTERN", [PragmaArgument.from_identifier(ChooseRandomRealSubRegions.NAME)], ChooseRandomRealSubRegions.build_signature())
    >>> program.add_instruction(Instruction.from_pragma(pragma_extern))
    >>> call = Call(ChooseRandomRealSubRegions.NAME, [
    ...     CallArgument.from_identifier("destination"),
    ...     CallArgument.from_identifier("source"),
    ...     CallArgument.from_immediate(complex(3, 0)),
    ...     CallArgument.from_memory_reference(MemoryReference("seed", 0)),
    ... ])
    >>> program.add_instruction(Instruction.from_call(call))
    >>> print(program.to_quil())
    PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
    DECLARE destination REAL[3]
    DECLARE source REAL[9]
    DECLARE seed INTEGER[1]
    CALL choose_random_real_sub_regions destination source 3 seed[0]

    From there, you may reference the `destination` memory region in your pulse program.
    """

    NAME: str
    """
    The name of the extern call function, which may be used in `PRAGMA EXTERN` and `CALL` instructions.
    """

    @classmethod
    def build_signature(cls) -> str:
        """Build the Quil signature of the `PRAGMA EXTERN` instruction."""
        ...


def lfsr_v1_next(seed_value: PrngSeedValue) -> PrngSeedValue:
    """Given a seed value, return the next value in the LFSR v1 PRNG sequence."""
    ...


def choose_random_real_sub_region_indices(
    seed: PrngSeedValue,
    start_index: int,
    series_length: int,
    sub_region_count: int,
) -> List[int]: 
    """
    Given a seed value, the starting index and length of a pseudo-random series, and the number of
    sub-regions from which to choose, return a list of the sub-region indices that were chosen.

    The LFSR v1 pseudo-random number generator underlies this sequence.
    """
    ...


@final
class RandomizedMeasurementsError(ValueError): 
    """
    An error that can occur when adding randomized measurements to a program.
    """
    ...


@final
class UnitarySet:
    """
    A set of unitaries that may be applied to a pulse program prior to measurement. Currently,
    there is a single enum variant, `ZXZXZ`, which represents a unitary as the following
    sequence of instructions:

    RZ(angle_0)-RX(pi/2)-RZ(angle_1)-RX(pi/2)-RZ(angle_2)

    Each unitary is, thus, represented as the three angles which parameterize the RZ gates.
    """

    def is_zxzxz(self) -> bool: ...
    def as_zxzxz(self) -> Optional[NDArray[np.float64]]: ...
    def to_zxzxz(self) -> NDArray[np.float64]: ...
    @staticmethod
    def from_zxzxz(inner: NDArray[np.float64]) -> "UnitarySet":
        """
        Initialize a `UnitarySet` as a ZXZXZ decomposition. The input array must have shape (n, 3),
        where n is the number of unitaries in the set.
        """
        ...


@final
class RandomizedMeasurement:
    # TODO: should we make this signature more infallible?
    def __new__(cls, qubit: int, target: Optional[Tuple[str, int]]) -> Self: ...


@final
class RandomizedMeasurements:
    """A class that supports the addition of randomized measurements to a Quil program."""

    def __new__(cls, measurements: List[RandomizedMeasurement], unitary_set: UnitarySet, leading_delay: float = ...) -> Self: ...

    def append_to_program(self, target_program: str) -> str:
        """
        Given a target Quil program, this routine will add the necessary declarations, `PRAGMA EXTERN`, `CALL`, and
        `DELAY` instructions to the beginning of the program for randomly selecting a measurement basis.

        It will then apply the pseudo-randomly selected measurement basis (see `UnitarySet`) to each measured
        qubit before measuring the qubit.
        """
        ...

    def to_parameters(self, seed_values: Dict[int, PrngSeedValue]) -> Dict[str, List[float]]:
        """
        Given a map of fixed qubit indices to seed values, return a memory map that is necessary to
        realize a pseudo-random sequence of measurements according to the specified seeds.
        """
        ...

    def get_random_indices(self, seed_values: Dict[int, PrngSeedValue], shot_count: int) -> Dict[int, List[int]]:
        """
        Given a map of fixed qubit indices to seed values, return the pseudo-randomly selected measurement
        indices for each qubit according to the specified seeds.
        """
        ...
