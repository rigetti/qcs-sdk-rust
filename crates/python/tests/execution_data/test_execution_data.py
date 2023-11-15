from qcs_sdk.qpu import ReadoutValues, QPUResultData
from qcs_sdk.qvm import QVMResultData
from qcs_sdk import ResultData, RegisterData, RegisterMatrix
import numpy as np
from numpy.testing import assert_array_equal
import pytest


class TestResultData:
    def test_to_register_map_from_qpu_result_data(self):
        mappings = {
            "ro[0]": "qA",
            "ro[1]": "qB",
            "ro[2]": "qC",
        }
        values = {
            "qA": ReadoutValues.from_integer([0, 1]),
            "qB": ReadoutValues.from_integer([1, 2]),
            "qC": ReadoutValues.from_integer([2, 3]),
        }
        result_data = ResultData.from_qpu(QPUResultData(mappings, values))
        register_map = result_data.to_register_map()
        ro = register_map.get_register_matrix("ro")
        assert ro is not None, "'ro' should exist in the register map"
        ro = ro.as_integer()
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
            "qA": ReadoutValues.from_integer([0, 1]),
            "qB": ReadoutValues.from_integer([1]),
            "qC": ReadoutValues.from_integer([2, 3]),
        }
        result_data = ResultData.from_qpu(QPUResultData(mappings, values))

        with pytest.raises(ValueError):
            result_data.to_register_map()

    def test_to_register_map_from_qvm_result_data(self):
        qvm_memory_map = {"ro": RegisterData.from_i16([[0, 1, 2], [1, 2, 3]])}
        qvm_result_data = QVMResultData.from_memory_map(qvm_memory_map)
        result_data = ResultData.from_qvm(qvm_result_data)
        register_map = result_data.to_register_map()
        ro = register_map.get_register_matrix("ro")
        assert ro is not None, "'ro' should exist in the register map"
        ro = ro.as_integer()
        assert ro is not None, "'ro' should be an integer register matrix"
        expected = np.array([[0, 1, 2], [1, 2, 3]])

        assert_array_equal(ro, expected)


class TestRegisterMatrix:
    def test_integer(self):
        m = np.array([[0, 1, 2], [1, 2, 3]])
        register_matrix = RegisterMatrix.from_integer(m)
        assert register_matrix.is_integer()
        register_matrix = register_matrix.as_integer()
        assert register_matrix is not None, "register_matrix should be an integer matrix"
        assert_array_equal(register_matrix, m)

    def test_real(self):
        m = np.array([[0.0, 1.1, 2.2], [1.1, 2.2, 3.3]])
        register_matrix = RegisterMatrix.from_real(m)
        assert register_matrix.is_real()
        register_matrix = register_matrix.as_real()
        assert register_matrix is not None, "register_matrix should be a real matrix"
        assert_array_equal(register_matrix, m)

    def test_complex(self):
        m = np.array(
            [
                [complex(0, 1), complex(1, 2), complex(2, 3)],
                [complex(1, 2), complex(2, 3), complex(3, 4)],
            ]
        )
        register_matrix = RegisterMatrix.from_complex(m)
        assert register_matrix.is_complex()
        register_matrix = register_matrix.as_complex()
        assert register_matrix is not None, "register_matrix should be a complex matrix"
        assert_array_equal(register_matrix, m)


class TestRegisterMap:
    def test_iter(self):
        memory_map = {
            "ro": RegisterData.from_i16([[0, 1, 2], [1, 2, 3]]),
            "foo": RegisterData.from_i16([[0, 1, 2], [1, 2, 3]]),
        }
        qvm_result_data = QVMResultData.from_memory_map(memory_map)
        result_data = ResultData.from_qvm(qvm_result_data)
        register_map = result_data.to_register_map()
        expected_keys = {"ro", "foo"}
        actual_keys = set()
        for key, matrix in register_map.items():
            actual_keys.add(key)
            assert np.all(matrix.to_ndarray() == np.matrix([[0, 1, 2], [1, 2, 3]]))

        assert expected_keys == actual_keys == set(register_map.keys())
