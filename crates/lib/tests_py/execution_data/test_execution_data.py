from datetime import timedelta
import pickle

import pytest
import numpy as np
from numpy.testing import assert_array_equal

from qcs_sdk import QcsSdkError
from qcs_sdk.qpu import ReadoutValues, MemoryValues, QPUResultData
from qcs_sdk.qvm import QVMResultData
from qcs_sdk import ResultData, RegisterData, RegisterMatrix, ExecutionData


class TestResultData:
    def test_to_register_map_from_qpu_result_data(self):
        mappings = {
            "ro[0]": "qA",
            "ro[1]": "qB",
            "ro[2]": "qC",
        }
        readout_values = {
            "qA": ReadoutValues.Integer([0, 1]),
            "qB": ReadoutValues.Integer([1, 2]),
            "qC": ReadoutValues.Integer([2, 3]),
        }
        memory_values = {
            "ro": MemoryValues.Integer([1, 2, 3]),
        }
        result_data = ResultData.Qpu(QPUResultData(mappings, readout_values, memory_values))
        register_map = result_data.to_register_map()
        ro = register_map.get_register_matrix("ro")
        assert ro is not None, "'ro' should exist in the register map"
        match ro:
            case RegisterMatrix.Integer(ro):
                pass
            case _:
                pytest.fail(f"unexpected register matrix type: {type(ro)}")
        assert ro is not None, "'ro' should be an integer register matrix"
        expected = np.array([[0, 1, 2], [1, 2, 3]])

        assert_array_equal(ro, expected)

    def test_to_register_map_from_jagged_qpu_result_data(self):
        mappings = {
            "ro[0]": "qA",
            "ro[1]": "qB",
            "ro[2]": "qC",
        }
        values = {
            "qA": ReadoutValues.Integer([0, 1]),
            "qB": ReadoutValues.Integer([1]),
            "qC": ReadoutValues.Integer([2, 3]),
        }
        result_data = ResultData.Qpu(QPUResultData(mappings, values, {}))

        with pytest.raises(QcsSdkError):
            result_data.to_register_map()

    def test_to_register_map_from_qvm_result_data(self):
        qvm_memory_map = {"ro": RegisterData.I16([[0, 1, 2], [1, 2, 3]])}
        qvm_result_data = QVMResultData.from_memory_map(qvm_memory_map)
        result_data = ResultData.Qvm(qvm_result_data)
        register_map = result_data.to_register_map()
        ro = register_map.get_register_matrix("ro")
        assert ro is not None, "'ro' should exist in the register map"
        match ro:
            case RegisterMatrix.Integer(ro):
                pass
            case _:
                pytest.fail(f"unexpected register matrix type: {type(ro)}")
        assert ro is not None, "'ro' should be an integer register matrix"
        expected = np.array([[0, 1, 2], [1, 2, 3]])

        assert_array_equal(ro, expected)


@pytest.mark.parametrize(
    ("name", "m", "cls"),
    [
        ("integer", np.array([[0, 1, 2], [1, 2, 3]]), RegisterMatrix.Integer),
        ("real", np.array([[0.0, 1.1, 2.2], [1.1, 2.2, 3.3]]), RegisterMatrix.Real),
        ("complex",
         np.array([[complex(0, 1), complex(1, 2), complex(2, 3)],
                   [complex(1, 2), complex(2, 3), complex(3, 4)]]),
         RegisterMatrix.Complex),
    ]
)
def test_register_matrix(name: str, m: np.ndarray, cls: RegisterMatrix.Integer | RegisterMatrix.Real | RegisterMatrix.Complex):
    register_matrix = cls(m)
    match (name, register_matrix):
        case ("integer", RegisterMatrix.Integer(register_matrix)):
            pass
        case ("real", RegisterMatrix.Real(register_matrix)):
            pass
        case ("complex", RegisterMatrix.Complex(register_matrix)):
            pass
        case _:
            pytest.fail(f"unexpected register_matrix type: {type(register_matrix)}")

    assert register_matrix is not None, f"register_matrix should be an {name} matrix"
    assert_array_equal(register_matrix, m)

class TestRegisterMap:
    def test_iter(self):
        memory_map = {
            "ro": RegisterData.I16([[0, 1, 2], [1, 2, 3]]),
            "foo": RegisterData.I16([[0, 1, 2], [1, 2, 3]]),
        }
        qvm_result_data = QVMResultData.from_memory_map(memory_map)
        result_data = ResultData.Qvm(qvm_result_data)
        register_map = result_data.to_register_map()
        expected_keys = {"ro", "foo"}
        actual_keys = set()
        for key, matrix in register_map.items():
            actual_keys.add(key)
            assert np.all(matrix.to_ndarray() == np.array([[0, 1, 2], [1, 2, 3]]))

        assert expected_keys == actual_keys == set(register_map.keys())


class TestExecutionData:
    def test_pickle(self):
        """Test that ExecutionData can be pickled and unpickled."""
        qpu_result_data = QPUResultData(
            mappings={"q0": "ro"},
            readout_values={"q0": ReadoutValues([0, 1])},
            memory_values={"m": MemoryValues([0, 1])},
        )
        qvm_result_data = QVMResultData.from_memory_map({"ro": RegisterData([[0, 1]])})

        for execution_data in [
            ExecutionData(result_data=ResultData(qpu_result_data), duration=timedelta(seconds=1)),
            ExecutionData(result_data=ResultData(qvm_result_data), duration=None),
        ]:
            print(execution_data.result_data)
            pickled = pickle.dumps(execution_data)
            unpickled = pickle.loads(pickled)
            assert execution_data == unpickled
