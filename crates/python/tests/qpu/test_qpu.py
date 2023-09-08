import pytest

from qcs_sdk.qpu import (
    QPUResultData,
    ReadoutValues,
    ListQuantumProcessorsError,
    list_quantum_processors,
    list_quantum_processors_async,
)


def test_readout_values():
    inner = [0, 1]
    readout_values = ReadoutValues.from_integer(inner)
    assert readout_values.to_integer() == inner

    inner = [2.5, 3.5]
    readout_values = ReadoutValues.from_real(inner)
    assert readout_values.to_real() == inner

    inner = [4j, 5j]
    readout_values = ReadoutValues.from_complex(inner)
    assert readout_values.to_complex() == inner


def test_qpu_result_data():
    mappings = {"a": "_q0"}
    readout_values = {"a": ReadoutValues.from_integer([0, 1])}
    result_data = QPUResultData(mappings, readout_values)

    assert result_data.mappings == mappings
    assert result_data.readout_values["a"].as_integer() == readout_values["a"].as_integer()
    raw_data = result_data.to_raw_readout_data()
    assert raw_data.mappings == {"a": "_q0"}
    assert raw_data.readout_values == {"a": [0, 1]}


@pytest.mark.qcs_session
def test_list_quantum_processors():
    quantum_processor_ids = list_quantum_processors()
    assert len(quantum_processor_ids) > 0


def test_list_quantum_processors_timeout():
    with pytest.raises(ListQuantumProcessorsError, match="timeout"):
        list_quantum_processors(timeout=0)


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_list_quantum_processors_async():
    quantum_processor_ids = await list_quantum_processors_async()
    assert len(quantum_processor_ids) > 0
