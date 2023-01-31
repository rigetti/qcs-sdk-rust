from unittest.mock import patch
import pytest

import re

import qcs_sdk


@pytest.mark.asyncio
async def test_compile(native_bitflip_program: str, device_2q: str):
    await qcs_sdk.compile(native_bitflip_program, device_2q)


@pytest.mark.asyncio
@pytest.mark.skip
async def test_translate(native_bitflip_program: str, device_2q: str):
    await qcs_sdk.translate(native_bitflip_program, 1, "Aspen-11")


@pytest.mark.asyncio
@pytest.mark.skip
async def test_execute(bell_program: str, device_2q):
    compiled_program = await qcs_sdk.compile(bell_program, device_2q)
    translation_result = await qcs_sdk.translate(compiled_program, 1, "Aspen-11")
    job_id = await qcs_sdk.submit(translation_result["program"], {}, "Aspen-11")
    await qcs_sdk.retrieve_results(job_id, "Aspen-11")


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


@pytest.mark.asyncio
async def test_get_quilc_version():
    version = await qcs_sdk.get_quilc_version()
    assert re.match(r"^([0-9]+)\.([0-9]+)\.([0-9]+)$", version)


@pytest.mark.asyncio
@pytest.mark.skip
async def test_list_quantum_processors():
    qpus = await qcs_sdk.list_quantum_processors()
    assert isinstance(qpus, list)
