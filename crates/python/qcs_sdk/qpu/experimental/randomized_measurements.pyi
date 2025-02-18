"""
Measurement randomization is a technique used in both quantum tomography and
quantum error mitigation. Essentially, it involves randomly rotating each
qubit prior to measurement. This module enables per-shot randomization, where
random rotations are applied to each qubit independently for each shot. For
some background on the technique, see
[Predicting Many Properties of a Quantum System from Very Few Measurements
(arxiv:2002.08953)](https://arxiv.org/abs/2002.08953).

The [`RandomizedMeasurements`] struct handles three critical components to
correctly add randomized measurements to a Rigetti QCS Quil program:

1. Program construction - adding the classical randomization calls to the
    prologue of the program (i.e. before the pulse program begins) and referencing
    those randomized values within a unitary decomposition prior to measurement.
2. Parameter construction - building a map of [`Parameters`] with seeds for
    each qubit.
3. PRNG reconstruction - backing out the random indices that were drawn on each
    qubit during program execution.

This is not a QIS (quantum information science) library, but rather an
SDK for collecting data from Rigetti QPUs. As such, defining a proper
unitary set and using randomized measurement data is beyond the scope of this
library.

# Example

The below test module illustrates usage for implementing a `UnitarySet` and
the `RandomizedMeasurements` class using a RZ-RX-RZ-RX-RZ unitary decomposition.

```python
.. include:: tests/qpu/experimental/test_randomized_measurements.py
```
"""

from qcs_sdk.qpu.experimental.random import PrngSeedValue

from typing import Dict, List, Self, final

from qcs_sdk._unitary_set import UnitarySet as UnitarySet

@final
class RandomizedMeasurementsError(ValueError):
    """
    An error that can occur when adding randomized measurements to a program.
    """

    ...

@final
class AppendToProgramError(ValueError):
    """An error that can occur when appending randomized measurements to a program."""

    ...

@final
class ToParametersError(ValueError):
    """An error that can occur when converting randomized measurements to parameters."""

    ...

@final
class RandomizedMeasurement:
    """
    A Quil measurement instruction defined on a fixed qubit and specific memory reference.
    """

    def __new__(cls, qubit: int, memory_reference_name: str, memory_reference_index: int) -> Self: ...
    @property
    def qubit(self) -> int: ...
    @property
    def memory_reference_name(self) -> str: ...
    @property
    def memory_reference_index(self) -> int: ...

@final
class UnitaryParameterDeclaration:
    """
    A Quil declaration for a unitary parameter. The memory region is implicitly `REAL`. These
    declarations may represent the `UnitarySet` source or the destination of a
    `ChooseRandomRealSubRegions` call.
    """

    @property
    def name(self) -> str: ...
    @property
    def length(self) -> float: ...

@final
class QubitRandomization:
    """
    The declarations and measurements required to randomize a single qubit. This is passed to
    `UnitarySet.to_instructions` to add the necessary instructions for each qubit to a
    Quil program.

    The qubit may be accessed by the `measurement.qubit` property.
    """

    @property
    def seed_declaration(seed) -> UnitaryParameterDeclaration:
        """
        Reference to the declaration that will be used as the seed to the PRNG
        sequence for this qubit.
        """
        ...

    @property
    def destination_declaration(self) -> UnitaryParameterDeclaration:
        """
        Reference to the declaration that will hold the parameters that characterize
        the random rotation to be applied to this qubit on a per-shot basis.
        """
        ...

    @property
    def measurement(self) -> RandomizedMeasurement:
        """The measurement instruction that will be used to measure the qubit."""
        ...

@final
class RandomizedMeasurements:
    """A class that supports the addition of randomized measurements to a Quil program."""

    def __new__(
        cls, measurements: List[RandomizedMeasurement], unitary_set: UnitarySet, leading_delay: float = ...
    ) -> Self: ...
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
