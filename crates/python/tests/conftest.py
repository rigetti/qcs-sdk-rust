import os

import pytest

from qcs_sdk.qpu.isa import InstructionSetArchitecture

TEST_ROOT_DIR = os.path.dirname(os.path.realpath(__file__))
TEST_CONFIG_DIR = os.path.join(TEST_ROOT_DIR, "./_qcs_config")
TEST_FIXTURE_DIR = os.path.join(TEST_ROOT_DIR, "./_fixtures")

TEST_QCS_SETTINGS_PATH = os.path.join(TEST_CONFIG_DIR, "settings.toml")
TEST_QCS_SECRETS_PATH = os.path.join(TEST_CONFIG_DIR, "secrets.toml")
os.environ["QCS_SETTINGS_FILE_PATH"] = TEST_QCS_SETTINGS_PATH
os.environ["QCS_SECRETS_FILE_PATH"] = TEST_QCS_SECRETS_PATH

def _read_fixture(relpath: str) -> str:
    with open(os.path.join(TEST_FIXTURE_DIR, relpath)) as f:
        contents = f.read()
    return contents


@pytest.fixture
def quantum_processor_id() -> str:
    return "Aspen-M-3"


@pytest.fixture
def aspen_m_3_isa() -> InstructionSetArchitecture:
    return InstructionSetArchitecture.from_raw(_read_fixture("./aspen-m-3.json"))


@pytest.fixture
def device_2q() -> str:
    return _read_fixture("./device-2q.json")


@pytest.fixture
def native_bitflip_program() -> str:
    return """
DECLARE ro BIT[0]
RX(pi) 0
MEASURE 0 ro[0]
"""


@pytest.fixture
def bell_program() -> str:
    return """
DECLARE ro BIT[2]
X 0
CNOT 0 1
MEASURE 0 ro[0]
MEASURE 1 ro[1]
"""
