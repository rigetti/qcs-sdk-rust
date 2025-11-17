import pytest

from qcs_sdk import (
    ExecutionData,
    ResultData,
    RegisterMap,
    RegisterMatrix,
    Executable,
    ExeParameter,
    JobHandle,
    Service,
    RegisterData,
    ExecutionError,
    RegisterMatrixConversionError,
)
from qcs_sdk.qvm import QVMClient

@pytest.mark.qcs_session
@pytest.mark.qcs_execution
def test_execute_qpu(
    bell_program: str,
    quantum_processor_id: str,
):
    executable = Executable(bell_program, shots=1)
    results = executable.execute_on_qpu(quantum_processor_id)
    results = results.result_data.as_qpu()

    key = results.mappings["ro[0]"]
    shots = results.readout_values.get(key).as_integer()
    shot_value = shots[0]

    assert shot_value in [ 0, 1 ]


@pytest.mark.qcs_session
@pytest.mark.qcs_execution
def test_submit_and_retrieve_qpu(
    bell_program: str,
    quantum_processor_id: str,
):
    executable = Executable(bell_program, shots=1)
    job_handle = executable.submit_to_qpu(quantum_processor_id)

    assert list(job_handle.readout_map.keys()) == ["ro[0]", "ro[1]"]

    results = executable.retrieve_results(job_handle).result_data.as_qpu()

    key = results.mappings["ro[0]"]
    shots = results.readout_values.get(key).as_integer()
    shot_value = shots[0]

    assert shot_value in [ 0, 1 ]


def test_execute_qvm(
    bell_program: str,
    qvm_http_client: QVMClient,
):
    executable = Executable(bell_program, shots=1)
    results = executable.execute_on_qvm(qvm_http_client)
    results = results.result_data.as_qvm()

    vals = results.memory["ro"]
    shot = vals.as_i8()[0]
    shot_value = shot[0]

    assert shot_value in [ 0, 1 ]
