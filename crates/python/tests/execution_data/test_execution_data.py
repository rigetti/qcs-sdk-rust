from qcs_sdk.qpu import ReadoutValues, QPUReadout
from qcs_sdk import ResultData, RegisterData, RegisterMatrix
import numpy as np
from numpy.testing import assert_array_equal
import pytest


class TestResultData:
    def test_to_register_map_from_qpu_readout(self):
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
        readout_data = ResultData.from_qpu(QPUReadout(mappings, values))
        readout_map = readout_data.to_register_map()
        ro = readout_map.get_register_matrix("ro").as_integer()
        expected = np.array([[0, 1, 2], [1, 2, 3]])

        assert_array_equal(ro, expected)

    def test_to_register_map_from_jagged_qpu_readout(self):
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
        readout_data = ResultData.from_qpu(QPUReadout(mappings, values))

        with pytest.raises(ValueError):
            readout_data.to_register_map()

    def test_to_register_map_from_qvm_memory(self):
        qvm_memory_map = {"ro": RegisterData.from_i16([[0, 1, 2], [1, 2, 3]])}
        readout_data = ResultData.from_qvm(qvm_memory_map)
        readout_map = readout_data.to_register_map()
        ro = readout_map.get_register_matrix("ro").as_integer()
        expected = np.array([[0, 1, 2], [1, 2, 3]])

        assert_array_equal(ro, expected)


class TestRegisterMatrix:
    def test_integer(self):
        m = np.array([[0, 1, 2], [1, 2, 3]])
        register_matrix = RegisterMatrix.from_integer(m)
        assert_array_equal(register_matrix.as_integer(), m)
        assert register_matrix.is_integer()

    def test_real(self):
        m = np.array([[0.0, 1.1, 2.2], [1.1, 2.2, 3.3]])
        register_matrix = RegisterMatrix.from_real(m)
        assert_array_equal(register_matrix.as_real(), m)
        assert register_matrix.is_real()

    def test_complex(self):
        m = np.array(
            [
                [complex(0, 1), complex(1, 2), complex(2, 3)],
                [complex(1, 2), complex(2, 3), complex(3, 4)],
            ]
        )
        register_matrix = RegisterMatrix.from_complex(m)
        assert_array_equal(register_matrix.as_complex(), m)
        assert register_matrix.is_complex()
