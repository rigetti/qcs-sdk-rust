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
    destination = Declaration("destination", Vector(ScalarType.REAL, 3), None)
    program.add_instruction(Instruction.Declaration(destination))
    source = Declaration("source", Vector(ScalarType.REAL, 12), None)
    program.add_instruction(Instruction.Declaration(source))
    seed = Declaration("seed", Vector(ScalarType.INTEGER, 1), None)
    program.add_instruction(Instruction.Declaration(seed))
    pragma_extern = Pragma(
        "EXTERN",
        [PragmaArgument.Identifier(ChooseRandomRealSubRegions.NAME)],
        ChooseRandomRealSubRegions.build_signature(),
    )
    program.add_instruction(Instruction.Pragma(pragma_extern))
    call = Call(
        ChooseRandomRealSubRegions.NAME,
        [
            CallArgument.Identifier("destination"),
            CallArgument.Identifier("source"),
            CallArgument.Immediate(complex(3, 0)),
            CallArgument.MemoryReference(MemoryReference("seed", 0)),
        ],
    )
    program.add_instruction(Instruction.Call(call))
    assert program == Program.parse(EXPECTED_QUIL)
