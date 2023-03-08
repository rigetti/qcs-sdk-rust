import pytest

from qcs_sdk.qpu.isa import InstructionSetArchitecture
from qcs_sdk.compiler.quilc import (
    DEFAULT_COMPILER_TIMEOUT,
    CompilerOpts,
    TargetDevice,
    QuilcError,
    compile_program,
    compile_program_async,
    get_version_info,
    get_version_info_async,
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
    isa.benchmarks=[
        benchmark for benchmark in isa.benchmarks
        if benchmark.name != "randomized_benchmark_simultaneous_1q"
    ]
    with pytest.raises(QuilcError):
        TargetDevice.from_isa(isa)


def test_compile_program(
    bell_program: str,
    target_device: TargetDevice,
):
    """A simple program should compile successfully."""
    compiled_program = compile_program(bell_program, target_device)
    assert compiled_program


@pytest.mark.asyncio
async def test_compile_program_async(
    bell_program: str,
    target_device: TargetDevice,
):
    """A simple program should compile successfully."""
    compiled_program = await compile_program_async(bell_program, target_device)
    assert compiled_program


def test_get_version_info():
    """A valid version should be returned."""
    version = get_version_info()
    assert version


@pytest.mark.asyncio
async def test_get_version_info_async():
    """A valid version should be returned."""
    version = await get_version_info_async()
    assert version
