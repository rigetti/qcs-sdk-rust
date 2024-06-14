import os
from typing import List

import pytest
from _pytest.config import Config
from _pytest.config.argparsing import Parser
from _pytest.nodes import Item

from qcs_sdk.client import QCSClient
from qcs_sdk.qpu.isa import InstructionSetArchitecture
from qcs_sdk.qvm import QVMClient
from qcs_sdk.compiler.quilc import QuilcClient


TEST_ROOT_DIR = os.path.dirname(os.path.realpath(__file__))
TEST_CONFIG_DIR = os.path.join(TEST_ROOT_DIR, "./_qcs_config")
TEST_FIXTURE_DIR = os.path.join(TEST_ROOT_DIR, "./_fixtures")


def pytest_addoption(parser: Parser):
    parser.addoption("--with-qcs-session", action="store_true", default=False, help="Run tests that require proper user config authentication.")
    parser.addoption("--with-qcs-execution", action="store_true", default=False, help="Run tests that require qpu execution.")


def pytest_configure(config: Config):
    config.addinivalue_line("markers", "qcs_session: mark test as requiring authentication + authorization.")
    config.addinivalue_line("markers", "not_qcs_session: mark test as requiring no authentication + authorization.")
    config.addinivalue_line("markers", "qcs_execution: mark test as requiring qpu execution.")


def pytest_collection_modifyitems(config: Config, items: List[Item]):
    with_qcs_session = config.getoption("--with-qcs-session")
    with_qcs_execution = config.getoption("--with-qcs-execution")

    if not with_qcs_session:
        os.environ["QCS_SETTINGS_FILE_PATH"] = os.path.join(TEST_CONFIG_DIR, "settings.toml")
        os.environ["QCS_SECRETS_FILE_PATH"] = os.path.join(TEST_CONFIG_DIR, "secrets.toml")

    skip_not_sess = pytest.mark.skip(reason="requires --with-qcs-session pytest option to be false.")
    skip_sess = pytest.mark.skip(reason="requires --with-qcs-session pytest option to be true.")
    skip_exec = pytest.mark.skip(reason="requires --with-qcs-execution pytest option to be true.")

    for item in items:
        if not with_qcs_session:
            if "qcs_session" in item.keywords:
                item.add_marker(skip_sess)
        else:
            if "not_qcs_session" in item.keywords:
                item.add_marker(skip_not_sess)

        if not with_qcs_execution:
            if "qcs_execution" in item.keywords:
                item.add_marker(skip_exec)


def _read_fixture(relpath: str) -> str:
    with open(os.path.join(TEST_FIXTURE_DIR, relpath)) as f:
        contents = f.read()
    return contents


@pytest.fixture
def quantum_processor_id() -> str:
    return os.getenv("TEST_LIVE_QUANTUM_PROCESSOR_ID", "Aspen-M-3") 


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


@pytest.fixture
def qvm_http_client() -> QVMClient:
    return QVMClient.new_http(QCSClient.load().qvm_url)


@pytest.fixture
def quilc_rpcq_client() -> QuilcClient:
    return QuilcClient.new_rpcq(QCSClient.load().quilc_url)


@pytest.fixture(scope="session")
def live_qpu_access(request: pytest.FixtureRequest) -> bool:
    return (
        request.config.getoption("--with-qcs-execution") is not None
        and request.config.getoption("--with-qcs-execution") is not False
    )

