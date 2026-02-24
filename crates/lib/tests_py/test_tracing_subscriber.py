"""Test traced methods.

This module has a session-scoped fixture that initializes the Rust tracing subscriber,
and individual tests record expected Rust spans into the collection that fixture provides.
Verification of those expectations is performed after the Rust `Tracing` context manager
shuts down, as that's the only time we can guarantee that the trace file is flushed.

As a result, individual tests only validate that the traced Rust calls succeed.
Validation of the actual span values happens all at once,
and so its failure may represent a failure of the entire test suite.
It's an unfortunately compromise that we have to make
due to the global, only-can-be-initialized-once-per-process nature of the Rust tracing subscriber.
The only other reasonable alternative is to have a separate process for each test,
which is also unwieldy.

If the environment variable `QCS_SDK_TESTS_KEEP_RUST_TRACES` is set to `1`,
the traces will not be deleted after the test completes.
"""
from collections import Counter
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
import json
import os
from unittest import mock
from typing import Generator
import pytest

from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import (
    BatchSpanProcessor,
    ConsoleSpanExporter,
)

from qcs_sdk import Executable
from qcs_sdk.compiler.quilc import QuilcClient, TargetDevice, compile_program_async, compile_program
from qcs_sdk.qpu.api import retrieve_results, retrieve_results_async, submit, submit_async
from qcs_sdk.qpu.isa import InstructionSetArchitecture
from qcs_sdk.qpu.translation import (
    translate,
    translate_async,
)
from qcs_sdk.qvm import QVMClient
from qcs_sdk.qvm.api import MultishotRequest, run_async as run_qvm_async, run as run_qvm
from qcs_sdk._tracing_subscriber import GlobalTracingConfig, SimpleConfig, Tracing, subscriber, layers


DIR = os.path.dirname(os.path.abspath(__file__))
TRACE_DIR = os.path.join(DIR, "__output__", "traces")


@dataclass(frozen=True)
class _ExpectedSpan:
    added_by: str
    trace_id: int
    span_name: str
    count: int
    minimum_expected_duration: timedelta
    parent_span_name: str


def test_quilc_tracing_sync(
    native_bitflip_program: str,
    tracer: trace.Tracer,
    expected_spans: list[_ExpectedSpan],
    request: pytest.FixtureRequest,
    aspen_m_3_isa: InstructionSetArchitecture,
    quilc_rpcq_client: QuilcClient,
):
    """
    Ensure that quilc `compile_program` is traced. This is a convenient unit test to ensure
    that the basic tracing setup is working.

    Note: this test records expected Rust spans; verification is performed after the Rust
    `Tracing` context manager shuts down in the `expected_spans` fixture teardown.
    """

    python_spans = []
    isa = TargetDevice.from_isa(aspen_m_3_isa)
    with tracer.start_as_current_span("test_quilc_tracing") as span:
        python_spans.append(span)
        compile_program(native_bitflip_program, isa, quilc_rpcq_client, None)

    for span in python_spans:
        expected_spans.append(
            _ExpectedSpan(
                added_by=request.node.nodeid,
                trace_id=span.get_span_context().trace_id,
                span_name="py_compile_program",
                count=1,
                minimum_expected_duration=timedelta(microseconds=10),
                parent_span_name=span.name,
            )
        )


@pytest.mark.asyncio
async def test_quilc_tracing_async(
    native_bitflip_program: str,
    tracer: trace.Tracer,
    expected_spans: list[_ExpectedSpan],
    request: pytest.FixtureRequest,
    aspen_m_3_isa: InstructionSetArchitecture,
    quilc_rpcq_client: QuilcClient,
):
    """Async analog of `test_quilc_tracing_sync`.

    Note: this test records expected Rust spans; verification is performed after the Rust
    `Tracing` context manager shuts down in the `expected_spans` fixture teardown.
    """

    python_spans = []
    isa = TargetDevice.from_isa(aspen_m_3_isa)
    with tracer.start_as_current_span("test_quilc_tracing_async") as span:
        python_spans.append(span)
        await compile_program_async(native_bitflip_program, isa, quilc_rpcq_client, None)

    for span in python_spans:
        expected_spans.append(
            _ExpectedSpan(
                added_by=request.node.nodeid,
                trace_id=span.get_span_context().trace_id,
                span_name="py_compile_program_async",
                count=1,
                minimum_expected_duration=timedelta(microseconds=10),
                parent_span_name=span.name,
            )
        )


def test_qvm_tracing_sync(
    native_bitflip_program: str,
    tracer: trace.Tracer,
    expected_spans: list[_ExpectedSpan],
    request: pytest.FixtureRequest,
    qvm_http_client: QVMClient,
):
    """
    Ensure that qvm `run` is traced. This is a convenient unit test to ensure
    that the basic tracing setup is working.

    Note: this test records expected Rust spans; verification is performed after the Rust
    `Tracing` context manager shuts down in the `expected_spans` fixture teardown.
    """

    python_spans = []
    multishot_request = MultishotRequest(
        program=native_bitflip_program,
        shots=10,
        addresses={},
        measurement_noise=None,
        gate_noise=None,
        rng_seed=None
    )
    with tracer.start_as_current_span("test_qvm_tracing") as span:
        python_spans.append(span)
        run_qvm(
            request=multishot_request,
            client=qvm_http_client,
        )

    for span in python_spans:
        expected_spans.append(
            _ExpectedSpan(
                added_by=request.node.nodeid,
                trace_id=span.get_span_context().trace_id,
                span_name="py_run",
                count=1,
                minimum_expected_duration=timedelta(microseconds=10),
                parent_span_name=span.name,
            )
        )


@pytest.mark.asyncio
async def test_qvm_tracing_async(
    native_bitflip_program: str,
    tracer: trace.Tracer,
    expected_spans: list[_ExpectedSpan],
    request: pytest.FixtureRequest,
    qvm_http_client: QVMClient,
):
    """Async analog of `test_qvm_tracing_sync`.

    Note: this test records expected Rust spans; verification is performed after the Rust
    `Tracing` context manager shuts down in the `expected_spans` fixture teardown.
    """

    python_spans = []
    multishot_request = MultishotRequest(
        program=native_bitflip_program,
        shots=10,
        addresses={},
        measurement_noise=None,
        gate_noise=None,
        rng_seed=None
    )
    with tracer.start_as_current_span("test_qvm_tracing_async") as span:
        python_spans.append(span)
        await run_qvm_async(
            request=multishot_request,
            client=qvm_http_client,
        )

    for span in python_spans:
        expected_spans.append(
            _ExpectedSpan(
                added_by=request.node.nodeid,
                trace_id=span.get_span_context().trace_id,
                span_name="py_run_async",
                count=1,
                minimum_expected_duration=timedelta(microseconds=10),
                parent_span_name=span.name,
            )
        )


@pytest.mark.qcs_session
def test_translate_submit_retrieve_sync(
    native_bitflip_program: str,
    quantum_processor_id: str,
    tracer: trace.Tracer,
    expected_spans: list[_ExpectedSpan],
    request: pytest.FixtureRequest,
    live_qpu_access: bool,
):
    """
    Run translation, job submission, and result retrieval using sync methods.

    Note: this test records expected Rust spans; verification is performed after the Rust
    `Tracing` context manager shuts down in the `expected_spans` fixture teardown.
    """

    python_spans = []
    with tracer.start_as_current_span("translate_submit_retrieve") as span:
        python_spans.append(span)
        translated = translate(native_bitflip_program, 1, quantum_processor_id)
        assert translated.program is not None
        if live_qpu_access:
            job_id = submit(translated.program, patch_values={}, quantum_processor_id=quantum_processor_id)
            retrieve_results(job_id, quantum_processor_id=quantum_processor_id)

    if live_qpu_access:
        with tracer.start_as_current_span("execute_on_qpu") as span:
            python_spans.append(span)
            executable = Executable(native_bitflip_program, shots=1)
            executable.execute_on_qpu(quantum_processor_id)

    for span in python_spans:
        expected_spans.append(
            _ExpectedSpan(
                added_by=request.node.nodeid,
                trace_id=span.get_span_context().trace_id,
                span_name=_TRANSLATE_QUIL_TO_ENCRYPTED_CONTROLLER_JOB,
                count=1,
                minimum_expected_duration=_MIN_EXPECTED_GRPC_DURATION,
                parent_span_name=span.name,
            )
        )
        if live_qpu_access:
            expected_spans.append(
                _ExpectedSpan(
                    added_by=request.node.nodeid,
                    trace_id=span.get_span_context().trace_id,
                    span_name=_EXECUTE_CONTROLLER_JOB,
                    count=1,
                    minimum_expected_duration=_MIN_EXPECTED_GRPC_DURATION,
                    parent_span_name=span.name,
                )
            )
            expected_spans.append(
                _ExpectedSpan(
                    added_by=request.node.nodeid,
                    trace_id=span.get_span_context().trace_id,
                    span_name=_GET_CONTROLLER_JOB_RESULTS,
                    count=1,
                    minimum_expected_duration=_MIN_EXPECTED_GRPC_DURATION,
                    parent_span_name=span.name,
                )
            )
@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_translate_submit_retrieve_async(
    native_bitflip_program: str,
    quantum_processor_id: str,
    tracer: trace.Tracer,
    expected_spans: list[_ExpectedSpan],
    request: pytest.FixtureRequest,
    live_qpu_access: bool,
):
    """
    Run translation, job submission, and result retrieval using async methods.

    Note: this test records expected Rust spans; verification is performed after the Rust
    `Tracing` context manager shuts down in the `expected_spans` fixture teardown.
    """

    python_spans = []
    with tracer.start_as_current_span("translate_submit_retrieve_async") as span:
        python_spans.append(span)
        translated = await translate_async(native_bitflip_program, 1, quantum_processor_id)
        assert translated.program is not None
        if live_qpu_access:
            job_id = await submit_async(translated.program, patch_values={}, quantum_processor_id=quantum_processor_id)
            await retrieve_results_async(job_id, quantum_processor_id=quantum_processor_id)

    if live_qpu_access:
        with tracer.start_as_current_span("execute_on_qpu_async") as span:
            python_spans.append(span)
            executable = Executable(native_bitflip_program, shots=1)
            await executable.execute_on_qpu_async(quantum_processor_id)

    for span in python_spans:
        expected_spans.append(
            _ExpectedSpan(
                added_by=request.node.nodeid,
                trace_id=span.get_span_context().trace_id,
                span_name=_TRANSLATE_QUIL_TO_ENCRYPTED_CONTROLLER_JOB,
                count=1,
                minimum_expected_duration=_MIN_EXPECTED_GRPC_DURATION,
                parent_span_name=span.name,
            )
        )
        if live_qpu_access:
            expected_spans.append(
                _ExpectedSpan(
                    added_by=request.node.nodeid,
                    trace_id=span.get_span_context().trace_id,
                    span_name=_EXECUTE_CONTROLLER_JOB,
                    count=1,
                    minimum_expected_duration=_MIN_EXPECTED_GRPC_DURATION,
                    parent_span_name=span.name,
                )
            )
            expected_spans.append(
                _ExpectedSpan(
                    added_by=request.node.nodeid,
                    trace_id=span.get_span_context().trace_id,
                    span_name=_GET_CONTROLLER_JOB_RESULTS,
                    count=1,
                    minimum_expected_duration=_MIN_EXPECTED_GRPC_DURATION,
                    parent_span_name=span.name,
                )
            )


_TRANSLATE_QUIL_TO_ENCRYPTED_CONTROLLER_JOB = "/services.translation.Translation/TranslateQuilToEncryptedControllerJob"
"""The gRPC method for translation."""

_EXECUTE_CONTROLLER_JOB = "/services.controller.Controller/ExecuteControllerJob"
"""The gRPC method for execution."""

_GET_CONTROLLER_JOB_RESULTS = "/services.controller.Controller/GetControllerJobResults"
"""The gRPC method for result retrieval."""

_MIN_EXPECTED_GRPC_DURATION = timedelta(milliseconds=10)
"""
The minimum amount of time we would expect a gRPC call to take. Note, this is a fairly
low duration, but it is sufficient to presume the network call was properly traced and that
the translation span is not empty.
"""


def _verify_expected_spans(rust_trace_file: str, expected_spans: list[_ExpectedSpan]):
    with open(rust_trace_file) as f:
        resource_spans = []
        for line in f.readlines():
            resource_spans += json.loads(line)["resourceSpans"]

    for expected in expected_spans:
        counter: Counter[str] = Counter()
        for resource_span in resource_spans:
            for scope_span in resource_span["scopeSpans"]:
                for span in scope_span["spans"]:
                    trace_id = int(span["traceId"], 16)
                    if trace_id == expected.trace_id:
                        counter[span["name"]] += 1
                        if span["name"] == expected.span_name:
                            duration_ns = int(span["endTimeUnixNano"]) - int(span["startTimeUnixNano"])
                            duration = timedelta(microseconds=duration_ns / 1000)
                            assert duration >= expected.minimum_expected_duration

        assert (
            counter[expected.span_name] == expected.count
        ), f'Expected {expected.count} spans with name {expected.span_name} within parent "{expected.parent_span_name}", but found {counter[expected.span_name]}. Added by {expected.added_by}. See {rust_trace_file} trace {hex(expected.trace_id)}'


@pytest.fixture(scope="module")
def expected_spans() -> Generator[list[_ExpectedSpan], None, None]:
    """Get a collection to which tests can add expected spans.

    Spans are verified after the `Tracing` context manager shuts down,
    i.e., after the test suite runs.

    This module fixture is initialized only once. This is necessary because the Rust tracing
    subscriber is a global singleton that will panic if initialized more than once within a
    single process.

    Note: individual tests only validate that the traced Rust calls succeed and record expected
    spans. Verification of those spans is delayed until after the `Tracing` context manager
    shuts down (fixture teardown), because the OTLP file exporter may not flush until shutdown.

    The trace file is deleted after the test is run,
    unless the environment variable `QCS_SDK_TESTS_KEEP_RUST_TRACES` is set to `1`.
    """
    timestamp = datetime.now().astimezone(timezone.utc).isoformat()
    file_name = f"rust-traces-{timestamp}.json"
    expected_spans: list[_ExpectedSpan] = []
    try:
        os.makedirs(TRACE_DIR, exist_ok=True)
        rust_trace_file = os.path.join(TRACE_DIR, file_name)

        layer = layers.otel_otlp_file.Config(file_path=rust_trace_file)
        sub = subscriber.Config(layer=layer)
        config = GlobalTracingConfig(export_process=SimpleConfig(subscriber=sub))

        with Tracing(config=config):
            yield expected_spans

        _verify_expected_spans(rust_trace_file, expected_spans)
    finally:
        if os.environ.get("QCS_SDK_TESTS_KEEP_RUST_TRACES", "false").lower() not in {"1", "true", "t"}:
            os.remove(os.path.join(TRACE_DIR, file_name))


@pytest.fixture(scope="function")
def tracer() -> Generator[trace.Tracer, None, None]:
    """
    Create a Python tracer that writes traces to `/dev/null` (we are not interested in Python
    traces in this test module). Yield the tracer and force a flush after the test is run.
    """
    provider = TracerProvider()
    with open(os.devnull, "w") as f:
        processor = BatchSpanProcessor(ConsoleSpanExporter(out=f))
        provider.add_span_processor(processor)
        try:
            yield provider.get_tracer(__name__)
        finally:
            provider.force_flush()

@pytest.fixture(scope="module", autouse=True)
def tracing_environment_variables() -> Generator[None, None, None]:
    """
    Sets environment variables to set the desired `EnvFilter` for the OTLP file export layer.
    """
    with mock.patch.dict(
        os.environ,
        {
            "RUST_LOG": "qcs=info",
            "QCS_API_TRACING_ENABLED": "1",
            "QCS_API_PROPAGATE_OTEL_CONTEXT": "1",
        },
    ):
        yield


