import pickle
import pytest

from qcs_sdk.qpu.translation import translate

from qcs_sdk.qpu.api import (
    ConnectionStrategy,
    ExecutionOptions,
    retrieve_results,
    submit,
)


@pytest.mark.qcs_execution
def test_submit_retrieve(quantum_processor_id: str, execution_options: ExecutionOptions):
    """
    Test the full program submission and retrieval.
    """

    program = "DECLARE theta REAL; RZ(theta) 0"
    memory = { "theta": [0.5] }

    translated = translate(program, 1, quantum_processor_id)

    job_id = submit(program=translated.program, patch_values=memory, quantum_processor_id=quantum_processor_id)
    results = retrieve_results(
        job_id=job_id,
        quantum_processor_id=quantum_processor_id,
        execution_options=execution_options,
    )
    assert results is not None

class TestPickle():
    @pytest.mark.parametrize("strategy", [ConnectionStrategy.Gateway(), ConnectionStrategy.DirectAccess(), ConnectionStrategy.EndpointId("endpoint_id"), ConnectionStrategy.EndpointAddress("http://localhost:8080")])
    def test_connection_strategy(self, strategy: ConnectionStrategy):
        pickled = pickle.dumps(strategy)
        unpickled = pickle.loads(pickled)
        assert unpickled == strategy

    def test_execution_options(self):
        options = ExecutionOptions.default()
        pickled = pickle.dumps(options)
        unpickled = pickle.loads(pickled)
        assert unpickled == options
