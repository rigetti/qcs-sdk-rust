from typing import List
from qcs_sdk.qpu.experimental import PrngSeedValue, RandomizedMeasurements, RandomizedMeasurement, UnitarySet
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
DECLARE ro BIT[3]
DECLARE randomized_measurement_source REAL[36]
DECLARE randomized_measurement_destination_q0 REAL[3]
DECLARE randomized_measurement_seed_q0 INTEGER[1]
DECLARE randomized_measurement_destination_q1 REAL[3]
DECLARE randomized_measurement_seed_q1 INTEGER[1]
DECLARE randomized_measurement_destination_q2 REAL[3]
DECLARE randomized_measurement_seed_q2 INTEGER[1]

DELAY 0 1 2 1e-6

PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"

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


@pytest.fixture
def randomized_measurements() -> RandomizedMeasurements:
    """
    Returns an initialized instance of RandomizedMeasurements, where three qubits are measured
    and the unitary set is of length 12 and zero-initialized.
    """
    measurements = [RandomizedMeasurement(qubit, ("ro", qubit)) for qubit in range(3)]
    unitary_set = UnitarySet.from_zxzxz(np.zeros((12, 3)))
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
    randomized_measurements: RandomizedMeasurements, seed_values: List[PrngSeedValue]
):
    """Test that the randomized measurement parameters are correctly generated."""
    parameters = randomized_measurements.to_parameters(
        {qubit: seed_value for qubit, seed_value in enumerate(seed_values)}
    )

    expected_parameters = {
        # The unitary set was initialized with zeros, so the angles are all zero.
        "randomized_measurement_source": [0.0] * 36,
        "randomized_measurement_seed_q0": [463_692_700],
        "randomized_measurement_destination_q0": [0.0, 0.0, 0.0],
        "randomized_measurement_seed_q1": [733_101_278],
        "randomized_measurement_destination_q1": [0.0, 0.0, 0.0],
        "randomized_measurement_seed_q2": [925_742_198],
        "randomized_measurement_destination_q2": [0.0, 0.0, 0.0],
    }

    assert parameters == expected_parameters


def test_randomized_measurements_get_random_indices(
    randomized_measurements: RandomizedMeasurements, seed_values: List[PrngSeedValue]
):
    """Test that, given a set of seed values, the correct random indices are generated."""
    shot_count = 3
    random_indices = randomized_measurements.get_random_indices(
        {qubit: seed_value for qubit, seed_value in enumerate(seed_values)}, shot_count
    )

    expected_random_indices = {0: [0, 8, 1], 1: [1, 2, 1], 2: [5, 10, 5]}

    assert random_indices == expected_random_indices
