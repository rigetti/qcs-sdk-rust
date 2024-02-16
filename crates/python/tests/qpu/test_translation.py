import pytest

from qcs_sdk.qpu.translation import (
    TranslationError,
    translate,
    translate_async,
    get_quilt_calibrations,
    get_quilt_calibrations_async,
)


@pytest.mark.qcs_session
def test_translate(
    native_bitflip_program: str,
    quantum_processor_id: str,
):
    translated = translate(native_bitflip_program, 1, quantum_processor_id)
    assert translated.program


@pytest.mark.qcs_session
def test_translate_error(
    quantum_processor_id: str,
):
    with pytest.raises(TranslationError):
        translate("DECLARE --", 1, quantum_processor_id)


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_translate_async(
    native_bitflip_program: str,
    quantum_processor_id: str,
):
    translated = await translate_async(native_bitflip_program, 1, quantum_processor_id)
    assert translated.program


@pytest.mark.qcs_session
def test_get_quilt_calibrations(
    quantum_processor_id: str,
):
    program = get_quilt_calibrations(quantum_processor_id)
    assert program


@pytest.mark.qcs_session
def test_get_quilt_calibrations_error():
    with pytest.raises(TranslationError):
        get_quilt_calibrations("--")


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_get_quilt_calibrations(
    quantum_processor_id: str,
):
    program = await get_quilt_calibrations_async(quantum_processor_id)
    assert program


def test_translation_backend():
    from qcs_sdk.qpu.translation import TranslationBackend, TranslationOptions

    opts = TranslationOptions()
    assert opts.backend is None
    opts.use_backend_v1()
    assert opts.backend == TranslationBackend.V1
    opts.use_backend_v2()
    assert opts.backend == TranslationBackend.V2
