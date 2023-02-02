import pytest

from os import path
import json

from qcs_sdk import get_instruction_set_architecture
from qcs_sdk.qpu.isa import InstructionSetArchitecture, Family



def ignore_nones(value):
    """Recursively ignore `None` values, useful for comparing json serializations."""
    if isinstance(value, list):
        return [ignore_nones(x) for x in value if x is not None]
    elif isinstance(value, dict):
        return {
            key: ignore_nones(val)
            for key, val in value.items()
            if val is not None
        }
    else:
        return value


@pytest.fixture
def aspen_m_3_json() -> str:
    filepath = path.join(path.dirname(__file__), "fixtures/aspen-m-3.json" )
    with open(filepath) as f:
        contents = f.read()
    return contents


def test_isa_from_aspen_m_3_json(aspen_m_3_json: str):
    isa = InstructionSetArchitecture.from_raw(aspen_m_3_json)

    assert isa.architecture.family == Family.Aspen

    node_ids = {node.node_id for node in isa.architecture.nodes}
    assert len(node_ids) == 80

    assert json.loads(isa.json()) == ignore_nones(json.loads(aspen_m_3_json))


@pytest.mark.asyncio
@pytest.mark.skip
async def test_get_isa_aspen_m_3():
    isa = await get_instruction_set_architecture("Aspen-M-3")

    assert isa.architecture.family == Family.Aspen
