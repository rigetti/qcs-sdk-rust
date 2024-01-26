import pickle

from qcs_sdk import ExecutionData, RegisterData
from qcs_sdk.qpu import QPUResultData, ReadoutValues, MemoryValues
from qcs_sdk.qvm import QVMResultData


def test_pickle():
    """Test that ExecutionData can be pickled and unpickled."""
    qpu_result_data = QPUResultData(
        mappings={"q0": "ro"},
        readout_values={"q0": ReadoutValues([0, 1])},
        memory_values={"m": MemoryValues([0, 1])},
    )
    qvm_result_data = QVMResultData.from_memory_map({"ro": RegisterData([[0, 1]])})

    for execution_data in [
        ExecutionData(result_data=qpu_result_data, duration=1.0),
        ExecutionData(result_data=qvm_result_data, duration=None),
    ]:
        pickled = pickle.dumps(execution_data)
        unpickled = pickle.loads(pickled)
        assert execution_data == unpickled
