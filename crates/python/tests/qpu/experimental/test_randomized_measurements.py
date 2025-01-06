from dataclasses import dataclass
from typing import List

from numpy.typing import NDArray
from quil.expression import Expression
from quil.instructions import Fence, Gate, Instruction, MemoryReference, Qubit
from qcs_sdk.qpu.experimental.random import PrngSeedValue
from qcs_sdk.qpu.experimental.randomized_measurements import (
    QubitRandomization,
    RandomizedMeasurements,
    RandomizedMeasurement,
    UnitarySet,
)
import numpy as np
import pytest
from quil.program import Program

BASE_QUIL_PROGRM = """
DECLARE ro BIT[3]

H 0
H 1
H 2
"""

BASE_QUIL_PROGRAM_WITH_MEASUREMENTS = """
PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"

DECLARE ro BIT[3]
DECLARE randomized_measurement_source REAL[36]
DECLARE randomized_measurement_destination_q0 REAL[3]
DECLARE randomized_measurement_seed_q0 INTEGER[1]
DECLARE randomized_measurement_destination_q1 REAL[3]
DECLARE randomized_measurement_seed_q1 INTEGER[1]
DECLARE randomized_measurement_destination_q2 REAL[3]
DECLARE randomized_measurement_seed_q2 INTEGER[1]

DELAY 0 1 2 1e-6

CALL choose_random_real_sub_regions randomized_measurement_destination_q0 randomized_measurement_source 3 randomized_measurement_seed_q0
CALL choose_random_real_sub_regions randomized_measurement_destination_q1 randomized_measurement_source 3 randomized_measurement_seed_q1
CALL choose_random_real_sub_regions randomized_measurement_destination_q2 randomized_measurement_source 3 randomized_measurement_seed_q2

H 0
H 1
H 2

FENCE

RZ(2*pi*randomized_measurement_destination_q0[0]) 0
RX(pi/2) 0
RZ(2*pi*randomized_measurement_destination_q0[1]) 0

RZ(2*pi*randomized_measurement_destination_q1[0]) 1
RX(pi/2) 1
RZ(2*pi*randomized_measurement_destination_q1[1]) 1

RZ(2*pi*randomized_measurement_destination_q2[0]) 2
RX(pi/2) 2
RZ(2*pi*randomized_measurement_destination_q2[1]) 2

FENCE

RX(pi/2) 0
RZ(2*pi*randomized_measurement_destination_q0[2]) 0

RX(pi/2) 1
RZ(2*pi*randomized_measurement_destination_q1[2]) 1

RX(pi/2) 2
RZ(2*pi*randomized_measurement_destination_q2[2]) 2

FENCE

MEASURE 0 ro[0]
MEASURE 1 ro[1]
MEASURE 2 ro[2]
"""


RADIANS_PER_CYCLE = 2 * np.pi


@dataclass(frozen=True, kw_only=True)
class Zxzxz(UnitarySet):
    angles: NDArray[np.float64]

    def parameters_per_unitary(self) -> int:
        return 3

    def unitary_count(self) -> int:
        return self.angles.shape[0]

    def to_parameters(self) -> List[float]:
        """
        Note, we divide by 2 pi here as the Quil RZ instructions contain a factor of 2 pi.
        This helps us avoid multiplying the reference value by 2 pi on the control system.
        """
        return (self.angles / RADIANS_PER_CYCLE).flatten().tolist()

    def to_instructions(self, qubit_randomizations: List[QubitRandomization]) -> str:
        instructions = Program()
        instructions.add_instruction(Instruction.from_fence(Fence([])))
        for qubit_randomization in qubit_randomizations:
            instructions.add_instruction(
                _rz(
                    qubit_randomization.measurement.qubit,
                    MemoryReference(qubit_randomization.destination_declaration.name, 0),
                )
            )
            instructions.add_instruction(_rx_pi_over_2(qubit_randomization.measurement.qubit))
            instructions.add_instruction(
                _rz(
                    qubit_randomization.measurement.qubit,
                    MemoryReference(qubit_randomization.destination_declaration.name, 1),
                )
            )

        instructions.add_instruction(Instruction.from_fence(Fence([])))
        for qubit_randomization in qubit_randomizations:
            instructions.add_instruction(_rx_pi_over_2(qubit_randomization.measurement.qubit))
            instructions.add_instruction(
                _rz(
                    qubit_randomization.measurement.qubit,
                    MemoryReference(qubit_randomization.destination_declaration.name, 2),
                )
            )

        return instructions.to_quil()


def _rz(qubit: int, memory_reference: MemoryReference) -> Instruction:
    """
    Note,that we multiply the reference by 2 pi to avoid the need to multiply the reference
    value by 2 pi on the control system. Accordingly, we also divide the parameters by 2 pi.
    """
    cycles = Expression.from_number(complex(2)) * Expression.new_pi() * Expression.from_address(memory_reference)
    return Instruction.from_gate(Gate("RZ", [cycles], [Qubit.from_fixed(qubit)], []))


def _rx_pi_over_2(qubit: int) -> Instruction:
    return Instruction.from_gate(
        Gate("RX", [Expression.new_pi() / Expression.from_number(complex(2))], [Qubit.from_fixed(qubit)], [])
    )


@pytest.fixture
def unitary_set() -> Zxzxz:
    rng = np.random.default_rng(949_261_248)
    return Zxzxz(angles=rng.uniform(0, RADIANS_PER_CYCLE, (12, 3)))


@pytest.fixture
def randomized_measurements(unitary_set: UnitarySet) -> RandomizedMeasurements:
    """
    Returns an initialized instance of RandomizedMeasurements, where three qubits are measured
    and the unitary set is of length 12 and drawn uniformly at random from [0, 2 pi).
    """
    measurements = [RandomizedMeasurement(qubit, "ro", qubit) for qubit in range(3)]
    return RandomizedMeasurements(measurements, unitary_set, 1e-6)


@pytest.fixture
def seed_values() -> List[PrngSeedValue]:
    """
    Returns a list of valid seed values. These values are valid and correspond to test expectations,
    but are otherwise indeed random.
    """
    return [PrngSeedValue(463_692_700), PrngSeedValue(733_101_278), PrngSeedValue(925_742_198)]


def test_randomized_measurements_append_to_program(randomized_measurements: RandomizedMeasurements):
    """Test that the randomized measurements are correctly appended to a Quil program."""
    program_with_randomized_measurements = randomized_measurements.append_to_program(BASE_QUIL_PROGRM)
    assert Program.parse(program_with_randomized_measurements) == Program.parse(BASE_QUIL_PROGRAM_WITH_MEASUREMENTS)


def test_randomized_measurements_to_parameters(
    randomized_measurements: RandomizedMeasurements,
    seed_values: List[PrngSeedValue],
    unitary_set: Zxzxz,
):
    """Test that the randomized measurement parameters are correctly generated."""
    parameters = randomized_measurements.to_parameters(
        {qubit: seed_value for qubit, seed_value in enumerate(seed_values)}
    )

    source_parameters = (unitary_set.angles / RADIANS_PER_CYCLE).flatten().tolist()

    assert np.allclose(parameters["randomized_measurement_source"], source_parameters)
    del parameters["randomized_measurement_source"]

    expected_parameters = {
        "randomized_measurement_seed_q0": [463_692_700],
        "randomized_measurement_destination_q0": [0.0, 0.0, 0.0],
        "randomized_measurement_seed_q1": [733_101_278],
        "randomized_measurement_destination_q1": [0.0, 0.0, 0.0],
        "randomized_measurement_seed_q2": [925_742_198],
        "randomized_measurement_destination_q2": [0.0, 0.0, 0.0],
    }

    assert parameters == expected_parameters


def test_randomized_measurements_get_random_indices(
    randomized_measurements: RandomizedMeasurements,
    seed_values: List[PrngSeedValue],
):
    """Test that, given a set of seed values, the correct random indices are generated."""
    shot_count = 3
    random_indices = randomized_measurements.get_random_indices(
        {qubit: seed_value for qubit, seed_value in enumerate(seed_values)}, shot_count
    )

    expected_random_indices = {0: [0, 8, 1], 1: [1, 2, 1], 2: [5, 10, 5]}

    assert random_indices == expected_random_indices
