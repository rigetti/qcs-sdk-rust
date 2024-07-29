import pickle
import pytest

from qcs_sdk.qpu.translation import (
    translate,
)

from qcs_sdk.qpu.api import (
    ConnectionStrategy,
    ExecutionOptions,
    Register,
    retrieve_results,
    submit,
)

def test_register():
    """Register should accept setting and getting data correctly."""
    data = [0j, 1j, 2j]
    register = Register.from_complex32(data)
    assert register.as_complex32() == data
    assert register.to_complex32() == data
    assert register.as_i32() == None
    with pytest.raises(ValueError):
        register.to_i32()

    data = [3, 4, 5]
    register = Register.from_i32(data)
    assert register.as_complex32() == None
    with pytest.raises(ValueError):
        register.to_complex32()
    assert register.as_i32() == data
    assert register.as_i32() == data


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

    job_id = submit(program, memory, quantum_processor_id)
    results = retrieve_results(job_id)

class TestPickle():
    @pytest.mark.parametrize("strategy", [ConnectionStrategy.gateway(), ConnectionStrategy.direct_access(), ConnectionStrategy.endpoint_id("endpoint_id")])
    def test_connection_strategy(self, strategy: ConnectionStrategy):
        pickled = pickle.dumps(strategy)
        unpickled = pickle.loads(pickled)
        assert unpickled == strategy

    def test_execution_options(self):
        options = ExecutionOptions.default()
        pickled = pickle.dumps(options)
        unpickled = pickle.loads(pickled)
        assert unpickled == options
