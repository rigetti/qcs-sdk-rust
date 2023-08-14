from qcs_sdk import RegisterData
from qcs_sdk.qvm import QVMResultData


def test_qvm_result_data():
    register_data = RegisterData.from_i8([[1, 2, 3], [4, 5, 6]])
    result_data = QVMResultData({"ro": register_data})
    assert result_data.asdict() == {"memory": {"ro": [[1, 2, 3], [4, 5, 6]]}}
