import pytest

from qcs_sdk.compiler.quilc import (
    TargetDevice,
    compile_program,
    compile_program_async,
)
from qcs_sdk.qpu.rewrite_arithmetic import (
    rewrite_arithmetic,
    build_patch_values,
)
from qcs_sdk.qpu.translation import (
    translate,
    translate_async,
)

from qcs_sdk.qpu.api import (
    SubmissionError,
    RetrieveResultsError,
    Register,
    ExecutionResult,
    ExecutionResults,
    retrieve_results,
    retrieve_results_async,
    submit,
    submit_async,
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


@pytest.mark.skip
class TestSubmitRetrieve:
    """
    Test the full program submission and retrieval.
    """

    # TODO: test submit / retrieve
    # def test_submit_retrieve(
    #     bell_program: str,
    #     target_device: TargetDevice,
    #     quantum_processor_id: str,
    # ):
    #     # TODO
    #     pass

    # @pytest.mark.asyncio
    # async def test_submit_retrieve_async(
    #     bell_program: str,
    #     target_device: TargetDevice,
    #     quantum_processor_id: str,
    # ):
    #     # TODO
    #     pass
