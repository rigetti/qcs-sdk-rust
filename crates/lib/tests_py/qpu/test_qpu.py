import pytest

from qcs_sdk.qpu import (
    QPUResultData,
    ReadoutValues,
    MemoryValues,
    ListQuantumProcessorsError,
    list_quantum_processors,
    list_quantum_processors_async,
)


def test_readout_values():
    inner = [0, 1]
    readout_values = ReadoutValues.Integer(inner)
    assert readout_values._0 == inner

    inner = [2.5, 3.5]
    readout_values = ReadoutValues.Real(inner)
    assert readout_values._0 == inner

    inner = [4j, 5j]
    readout_values = ReadoutValues.Complex(inner)
    assert readout_values._0 == inner


def test_qpu_result_data():
    mappings = {"a": "_q0"}
    readout_values = {"a": ReadoutValues.Integer([0, 1])}
    memory_values = { "int": MemoryValues.Integer([2, 3]), "real": MemoryValues.Real([3.0, 4.0]), "binary": MemoryValues.Binary([0, 1]) }
    result_data = QPUResultData(mappings, readout_values, memory_values)

    assert result_data.mappings == mappings
    assert result_data.readout_values["a"]._0 == readout_values["a"]._0
    assert result_data.memory_values["int"]._0 == memory_values["int"]._0
    assert result_data.memory_values["binary"]._0 == memory_values["binary"]._0
    assert result_data.memory_values["real"]._0 == memory_values["real"]._0
    raw_data = result_data.to_raw_readout_data()
    assert raw_data.mappings == {"a": "_q0"}
    assert raw_data.readout_values == {"a": [0, 1]}
    assert raw_data.memory_values == {"int": [2, 3], "binary": [0, 1], "real": [3.0, 4.0]}


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
