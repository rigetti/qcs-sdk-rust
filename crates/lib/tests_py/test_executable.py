import pytest

from qcs_sdk import Executable
from qcs_sdk.compiler.quilc import QuilcClient
from qcs_sdk.qpu import QPUResultData
from qcs_sdk.qvm import QVMClient, QVMResultData

@pytest.mark.qcs_session
@pytest.mark.qcs_execution
def test_execute_qpu(
    bell_program: str,
    quantum_processor_id: str,
    quilc_rpcq_client: QuilcClient,
):
    executable = Executable(bell_program, shots=1, quilc_client=quilc_rpcq_client)
    execution_data = executable.execute_on_qpu(quantum_processor_id)
    results = execution_data.result_data
    assert isinstance(results, QPUResultData)

    key = results.mappings["ro[0]"]
    shots = results.readout_values[key].inner()
    shot_value = shots[0]

    assert shot_value in [ 0, 1 ]


@pytest.mark.asyncio
@pytest.mark.qcs_session
@pytest.mark.qcs_execution
async def test_execute_qpu_async(
    bell_program: str,
    quantum_processor_id: str,
    quilc_rpcq_client: QuilcClient,
):
    executable = Executable(bell_program, shots=1, quilc_client=quilc_rpcq_client)
    execution_data = await executable.execute_on_qpu_async(quantum_processor_id)
    results = execution_data.result_data
    assert isinstance(results, QPUResultData)

    key = results.mappings["ro[0]"]
    shots = results.readout_values[key].inner()
    shot_value = shots[0]

    assert shot_value in [ 0, 1 ]


@pytest.mark.qcs_session
@pytest.mark.qcs_execution
def test_submit_and_retrieve_qpu(
    bell_program: str,
    quantum_processor_id: str,
    quilc_rpcq_client: QuilcClient,
):
    executable = Executable(bell_program, shots=1, quilc_client=quilc_rpcq_client)
    job_handle = executable.submit_to_qpu(quantum_processor_id)

    keys = set(job_handle.readout_map.keys())
    for key in ["ro[0]", "ro[1]"]:
        assert key in keys

    results = executable.retrieve_results(job_handle).result_data
    assert isinstance(results, QPUResultData)

    key = results.mappings["ro[0]"]
    shots = results.readout_values[key].inner()
    shot_value = shots[0]

    assert shot_value in [ 0, 1 ]


@pytest.mark.asyncio
@pytest.mark.qcs_session
@pytest.mark.qcs_execution
async def test_submit_and_retrieve_qpu_async(
    bell_program: str,
    quantum_processor_id: str,
    quilc_rpcq_client: QuilcClient,
):
    executable = Executable(bell_program, shots=1, quilc_client=quilc_rpcq_client)
    job_handle = await executable.submit_to_qpu_async(quantum_processor_id)

    keys = set(job_handle.readout_map.keys())
    for key in ["ro[0]", "ro[1]"]:
        assert key in keys

    results = (await executable.retrieve_results_async(job_handle)).result_data
    assert isinstance(results, QPUResultData)

    key = results.mappings["ro[0]"]
    shots = results.readout_values[key].inner()
    shot_value = shots[0]

    assert shot_value in [ 0, 1 ]


def test_execute_qvm(
    bell_program: str,
    qvm_http_client: QVMClient,
):
    executable = Executable(bell_program, shots=1)
    results = executable.execute_on_qvm(qvm_http_client).result_data
    assert isinstance(results, QVMResultData)

    vals = results.memory["ro"]
    shot = vals.inner()[0]
    shot_value = shot[0]

    assert shot_value in [ 0, 1 ]


@pytest.mark.asyncio
async def test_execute_qvm_async(
    bell_program: str,
    qvm_http_client: QVMClient,
):
    executable = Executable(bell_program, shots=1)
    results = (await executable.execute_on_qvm_async(qvm_http_client)).result_data
    assert isinstance(results, QVMResultData)

    vals = results.memory["ro"]
    shot = vals.inner()[0]
    shot_value = shot[0]

    assert shot_value in [ 0, 1 ]
