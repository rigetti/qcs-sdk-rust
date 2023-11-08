import pytest
from syrupy.assertion import SnapshotAssertion

from qcs_sdk.qpu.isa import InstructionSetArchitecture
from qcs_sdk.compiler.quilc import (
    DEFAULT_COMPILER_TIMEOUT,
    CompilerOpts,
    TargetDevice,
    PauliTerm,
    GenerateRandomizedBenchmarkingSequenceResponse,
    RandomizedBenchmarkingRequest,
    ConjugateByCliffordRequest,
    ConjugatePauliByCliffordResponse,
    QuilcError,
    compile_program,
    compile_program_async,
    get_version_info,
    get_version_info_async,
    conjugate_pauli_by_clifford,
    conjugate_pauli_by_clifford_async,
    generate_randomized_benchmarking_sequence,
    generate_randomized_benchmarking_sequence_async,
    QuilcClient,
)


@pytest.fixture
def target_device(aspen_m_3_isa: InstructionSetArchitecture) -> TargetDevice:
    return TargetDevice.from_isa(aspen_m_3_isa)


def test_default_compiler_timeout():
    """Ensure this exists and is the correct type."""
    assert type(DEFAULT_COMPILER_TIMEOUT) is float


def test_compiler_opts():
    """Ensure CompilerOpts can be constructed with defaults."""
    assert type(CompilerOpts.default()) == CompilerOpts


def test_target_device_error(aspen_m_3_isa: InstructionSetArchitecture):
    """TargetDevice cannot be built when ISA lacks ``randomized_benchmark_simultaneous_1q`` benchmark."""
    isa = InstructionSetArchitecture.from_raw(aspen_m_3_isa.json())
    isa.benchmarks = [
        benchmark
        for benchmark in isa.benchmarks
        if benchmark.name != "randomized_benchmark_simultaneous_1q"
    ]
    with pytest.raises(QuilcError):
        TargetDevice.from_isa(isa)


def test_compile_program(
    bell_program: str,
    target_device: TargetDevice,
    snapshot: SnapshotAssertion,
    quilc_rpcq_client: QuilcClient,
):
    """A simple program should compile successfully."""
    result = compile_program(bell_program, target_device, client=quilc_rpcq_client, options=CompilerOpts(protoquil=True))
    assert result.program == snapshot
    assert result.native_quil_metadata == snapshot


@pytest.mark.asyncio
async def test_compile_program_async(
    bell_program: str,
    target_device: TargetDevice,
    quilc_rpcq_client: QuilcClient,
):
    """A simple program should compile successfully."""
    compiled_program = await compile_program_async(bell_program, target_device, client=quilc_rpcq_client)
    assert compiled_program


def test_get_version_info(quilc_rpcq_client: QuilcClient):
    """A valid version should be returned."""
    version = get_version_info(client=quilc_rpcq_client)
    assert version


@pytest.mark.asyncio
async def test_get_version_info_async(quilc_rpcq_client: QuilcClient):
    """A valid version should be returned."""
    version = await get_version_info_async(client=quilc_rpcq_client)
    assert version


def test_conjugate_pauli_by_clifford(quilc_rpcq_client: QuilcClient):
    """Pauli should be conjugated by clifford."""
    request = ConjugateByCliffordRequest(
        pauli=PauliTerm(indices=[0], symbols=["X"]), clifford="H 0"
    )
    response = conjugate_pauli_by_clifford(request, client=quilc_rpcq_client)
    assert type(response) == ConjugatePauliByCliffordResponse
    assert response.pauli == "Z"
    assert response.phase == 0


@pytest.mark.asyncio
async def test_conjugate_pauli_by_clifford_async(quilc_rpcq_client: QuilcClient):
    """Pauli should be conjugated by clifford."""
    request = ConjugateByCliffordRequest(
        pauli=PauliTerm(indices=[0], symbols=["X"]), clifford="H 0"
    )
    response = await conjugate_pauli_by_clifford_async(request, client=quilc_rpcq_client)
    assert type(response) == ConjugatePauliByCliffordResponse
    assert response.pauli == "Z"
    assert response.phase == 0


def test_generate_randomized_benchmark_sequence(quilc_rpcq_client: QuilcClient):
    """Random benchmark should run predictably."""
    request = RandomizedBenchmarkingRequest(
        depth=2,
        qubits=1,
        gateset=["X 0", "H 0"],
        seed=314,
        interleaver="Y 0",
    )
    response = generate_randomized_benchmarking_sequence(request, client=quilc_rpcq_client)
    assert type(response) == GenerateRandomizedBenchmarkingSequenceResponse
    assert response.sequence == [[1, 0], [0, 1, 0, 1], [1, 0]]


@pytest.mark.asyncio
async def test_generate_randomized_benchmark_sequence_async(quilc_rpcq_client: QuilcClient):
    """Random benchmark should run predictably."""
    request = RandomizedBenchmarkingRequest(
        depth=2,
        qubits=1,
        gateset=["X 0", "H 0"],
        seed=314,
        interleaver="Y 0",
    )
    response = await generate_randomized_benchmarking_sequence_async(request, client=quilc_rpcq_client)
    assert type(response) == GenerateRandomizedBenchmarkingSequenceResponse
    assert response.sequence == [[1, 0], [0, 1, 0, 1], [1, 0]]
