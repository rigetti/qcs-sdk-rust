from quil.program import Program
from quil.instructions import (
    Declaration,
    Instruction,
    Pragma,
    Call,
    CallArgument,
    MemoryReference,
    Vector,
    ScalarType,
    PragmaArgument,
)
from qcs_sdk.qpu.experimental.random import ChooseRandomRealSubRegions


EXPECTED_QUIL = """
PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
DECLARE destination REAL[3]
DECLARE source REAL[12]
DECLARE seed INTEGER[1]
CALL choose_random_real_sub_regions destination source 3 seed[0]
"""


def test_choose_random_real_subregions():
    """Test that `ChooseRandomRealSubRegions` is correctly added to a Quil progrram."""
    program = Program()
    destination = Declaration("destination", Vector(ScalarType.Real, 3), None)
    program.add_instruction(Instruction.from_declaration(destination))
    source = Declaration("source", Vector(ScalarType.Real, 12), None)
    program.add_instruction(Instruction.from_declaration(source))
    seed = Declaration("seed", Vector(ScalarType.Integer, 1), None)
    program.add_instruction(Instruction.from_declaration(seed))
    pragma_extern = Pragma(
        "EXTERN",
        [PragmaArgument.from_identifier(ChooseRandomRealSubRegions.NAME)],
        ChooseRandomRealSubRegions.build_signature(),
    )
    program.add_instruction(Instruction.from_pragma(pragma_extern))
    call = Call(
        ChooseRandomRealSubRegions.NAME,
        [
            CallArgument.from_identifier("destination"),
            CallArgument.from_identifier("source"),
            CallArgument.from_immediate(complex(3, 0)),
            CallArgument.from_memory_reference(MemoryReference("seed", 0)),
        ],
    )
    program.add_instruction(Instruction.from_call(call))
    assert program == Program.parse(EXPECTED_QUIL)
