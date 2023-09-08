from qcs_sdk import RegisterData
from qcs_sdk.qvm import QVMResultData


def test_qvm_result_data():
    register_data = RegisterData.from_i8([[1, 2, 3], [4, 5, 6]])
    raw_data = QVMResultData({"ro": register_data}).to_raw_readout_data()
    assert raw_data.memory == {"ro": [[1, 2, 3], [4, 5, 6]]}
