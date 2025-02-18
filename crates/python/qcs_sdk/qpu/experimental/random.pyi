"""
This module supports low-level primitives for randomization on Rigetti's QPUs.
"""

from typing import List, final
from typing_extensions import Self


__all__ = [
    "RandomError", 
    "PrngSeedValue", 
    "ChooseRandomRealSubRegions",
    "lfsr_v1_next",
    "choose_random_real_sub_region_indices",
]


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
    * `sub_region_size` - The size of each sub-region.
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

    ```python
    .. include:: tests/qpu/experimental/test_random.py
    ```

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

