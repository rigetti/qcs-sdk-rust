import pytest

from qcs_sdk.qpu.isa import (
    SerializeISAError,
    GetISAError,
    InstructionSetArchitecture,
    get_instruction_set_architecture,
    get_instruction_set_architecture_async,
)

@pytest.mark.qcs_session
def test_get_instruction_set_architecture(quantum_processor_id: str):
    """Successfully get a known public ISA."""
    isa = get_instruction_set_architecture(quantum_processor_id)
    assert type(isa) is InstructionSetArchitecture


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_get_instruction_set_architecture_async(quantum_processor_id: str):
    """Successfully get a known public ISA."""
    isa = await get_instruction_set_architecture_async(quantum_processor_id)
    assert type(isa) is InstructionSetArchitecture

@pytest.mark.qcs_session
def test_get_instruction_set_architecture_error():
    with pytest.raises(GetISAError):
        get_instruction_set_architecture("--")


def test_instruction_set_architecture_serialization_error():
    with pytest.raises(SerializeISAError):
        InstructionSetArchitecture.from_raw('{ "fail": true }')
