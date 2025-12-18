import pickle
import typing_extensions
import pytest

from qcs_sdk.qpu.translation import (
    translate,
)

from qcs_sdk.qpu import ConnectionStrategy

from qcs_sdk.qpu.api import (
    ExecutionOptions,
    Register,
    retrieve_results,
    submit,
)

def fn(b: bool) -> tuple[()]:
    return ()

@pytest.mark.parametrize(("name", "cls", "data"),
    [
        ("Complex32", Register.Complex32, [0j, 1j, 2j]),
        ("I32", Register.I32, [3, 4, 5]),
    ]
)
def test_register(name: str, cls: Register.Complex32 | Register.I32, data: list[complex | int]):
    """Register should accept setting and getting data correctly."""
    register = cls(data)
    match (name, register):
        case ("Complex32", Register.Complex32(d)):
            assert d == data
        case ("I32", Register.I32(d)):
            assert d == data
        case _:
            pytest.fail("Register did not match")


@pytest.mark.qcs_execution
def test_submit_retrieve(
    quantum_processor_id: str,
):
    """
    Test the full program submission and retrieval.
    """

    program = "DECLARE theta REAL; RZ(theta) 0"
    memory = { "theta": [0.5] }

    translated = translate(program, 1, quantum_processor_id)

    job_id = submit(translated.program, memory, quantum_processor_id)
    results = retrieve_results(job_id)
    assert results is not None

class TestPickle():
    @pytest.mark.parametrize("strategy", [ConnectionStrategy.Gateway(), ConnectionStrategy.DirectAccess(), ConnectionStrategy.EndpointId("endpoint_id")])
    def test_connection_strategy(self, strategy: ConnectionStrategy):
        pickled = pickle.dumps(strategy)
        unpickled = pickle.loads(pickled)
        assert unpickled == strategy

    def test_execution_options(self):
        options = ExecutionOptions.default()
        pickled = pickle.dumps(options)
        unpickled = pickle.loads(pickled)
        assert unpickled == options
