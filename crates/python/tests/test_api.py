from unittest.mock import patch
import pytest

import re

import qcs_sdk


def test_compile(native_bitflip_program: str, device_2q: str):
    qcs_sdk.compile(native_bitflip_program, device_2q)


@pytest.mark.skip
def test_translate(native_bitflip_program: str, device_2q: str):
    qcs_sdk.translate(native_bitflip_program, 1, "Aspen-M-3")

def test_translate_exe(bell_program: str):
    shots = 1234
    executable = qcs_sdk.Executable(bell_program, shots=shots)
    response = executable.execute_on_qvm()
    data = response.result_data.to_qvm().memory.get("ro").to_i8()

    assert data == [[1, 1] for _ in range(shots)]


@pytest.mark.skip
def test_execute(bell_program: str, device_2q):
    compiled_program = qcs_sdk.compile(bell_program, device_2q)
    translation_result = qcs_sdk.translate(compiled_program, 1, "Aspen-M-3")
    job_id = qcs_sdk.submit(translation_result["program"], {}, "Aspen-M-3")
    qcs_sdk.retrieve_results(job_id, "Aspen-M-3")


def test_rewrite_arithmetic():
    native_quil = "RX(2*theta[0]) 0"
    rewritten_arithmetic = qcs_sdk.rewrite_arithmetic(native_quil)
    expected = {
        "program": "DECLARE __SUBST REAL[1]\nRX(__SUBST[0]) 0\n",
        "recalculation_table": ["((2*theta[0])/6.283185307179586)"],
    }
    assert rewritten_arithmetic.program == expected["program"]
    assert rewritten_arithmetic.recalculation_table == expected["recalculation_table"]


def test_build_patch_values():
    memory = {"theta": [2.0]}
    recalculation_table = ["0.5*theta[0]"]
    expected = {"theta": [2.0], "__SUBST": [1.0]}
    patch_values = qcs_sdk.build_patch_values(recalculation_table, memory)
    assert patch_values == expected


def test_exe_parameters():
    """Should be able to construct and pass exe parameters"""
    exe_parameter = qcs_sdk.ExeParameter("a", 1, 2.5)
    assert exe_parameter.name == "a"
    assert exe_parameter.index == 1
    assert exe_parameter.value == 2.5

    qcs_sdk.Executable("quil", parameters=[exe_parameter])


def test_get_quilc_version():
    version = qcs_sdk.get_quilc_version()
    assert re.match(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$", version)


@pytest.mark.skip
def test_list_quantum_processors():
    qpus = qcs_sdk.list_quantum_processors()
    assert isinstance(qpus, list)
