from unittest.mock import patch
import pytest

import qcs_sdk


@pytest.mark.asyncio
@pytest.mark.skip
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
        "recalculation_table": ['((2*theta[0])/6.283185307179586)'],
    }
    assert rewritten_arithmetic == expected


def test_build_patch_values():
    memory = {"theta": [2.0]}
    recalculation_table = ["0.5*theta[0]"]
    expected = {"theta": [2.0], "__SUBST": [1.0]}
    patch_values = qcs_sdk.build_patch_values(recalculation_table, memory)
    assert patch_values == expected