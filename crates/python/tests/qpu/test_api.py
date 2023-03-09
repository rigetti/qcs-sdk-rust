import pytest

from qcs_sdk.qpu.rewrite_arithmetic import (
    rewrite_arithmetic,
    build_patch_values,
)
from qcs_sdk.qpu.translation import (
    translate,
)

from qcs_sdk.qpu.api import (
    Register,
    retrieve_results,
    submit,
)

def test_register():
    """Register should accept setting and getting data correctly."""
    data = [0j, 1j, 2j]
    register = Register.from_complex32(data)
    assert register.as_complex32() == data
    assert register.to_complex32() == data
    assert register.as_i32() == None
    with pytest.raises(ValueError):
        register.to_i32()

    data = [3, 4, 5]
    register = Register.from_i32(data)
    assert register.as_complex32() == None
    with pytest.raises(ValueError):
        register.to_complex32()
    assert register.as_i32() == data
    assert register.as_i32() == data


@pytest.mark.qcs_execution
def test_submit_retrieve(
    quantum_processor_id: str,
):
    """
    Test the full program submission and retrieval.
    """

    program = "DECLARE theta REAL; RZ(theta) 0"
    memory = { "theta": [0.5] }

    translated = translate(program, 1, quantum_processor_id)
    rewritten = rewrite_arithmetic(translated.program)
    patch_values = build_patch_values(rewritten.recalculation_table, memory)

    job_id = submit(rewritten.program, patch_values, quantum_processor_id)
    results = retrieve_results(job_id)
