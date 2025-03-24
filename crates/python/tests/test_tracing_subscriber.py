from collections import Counter
from datetime import datetime, timedelta, timezone
import json
import os
from unittest import mock
from typing import Generator
import pytest

from opentelemetry import trace
from opentelemetry.sdk.trace import Span, TracerProvider
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


@pytest.mark.asyncio
async def test_quilc_tracing(
    tracing_environment_variables: None,
    native_bitflip_program: str,
    tracer: trace.Tracer,
    rust_global_trace_file: str,
    aspen_m_3_isa: InstructionSetArchitecture,
    quilc_rpcq_client: QuilcClient,
):
    """
    Ensure that quilc `compile_program` is traced. This is a convenient unit test to ensure
    that the basic tracing setup is working.
    """

    python_spans = []
    isa = TargetDevice.from_isa(aspen_m_3_isa)
    with tracer.start_as_current_span("test_quilc_tracing") as span:
        python_spans.append(span)
        compile_program(native_bitflip_program, isa, quilc_rpcq_client, None)
        await compile_program_async(native_bitflip_program, isa, quilc_rpcq_client, None)

    for span in python_spans:
        # see `qcs_sdk_python::compiler::quilc::compile_program`
        for expected_span_name in ["py_compile_program", "py_compile_program_async"]:
            _verify_resource_spans(
                rust_global_trace_file,
                expected_span_name,
                span,
                minimum_expected_duration=timedelta(microseconds=10),
            )


@pytest.mark.asyncio
async def test_qvm_tracing(
    tracing_environment_variables: None,
    native_bitflip_program: str,
    tracer: trace.Tracer,
    rust_global_trace_file: str,
    qvm_http_client: QVMClient,
):
    """
    Ensure that qvm `run` is traced. This is a convenient unit test to ensure
    that the basic tracing setup is working.
    """

    python_spans = []
    request = MultishotRequest(
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
            request=request,
            client=qvm_http_client,
        )
        await run_qvm_async(
            request=request,
            client=qvm_http_client,
        )

    for span in python_spans:
        # See `qcs_sdk_python::qvm::qpi::run`
        for expected_span_name in ["py_run", "py_run_async"]:
            _verify_resource_spans(
                rust_global_trace_file,
                expected_span_name,
                span,
                minimum_expected_duration=timedelta(microseconds=10),
            )


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_translate_submit_retrieve(
    tracing_environment_variables: None,
    native_bitflip_program: str,
    quantum_processor_id: str,
    tracer: trace.Tracer,
    rust_global_trace_file: str,
    live_qpu_access: bool,
):
    """
    Run translation, job submission, and result retrieval using async and non-async methods.
    Assert the underlying gRPC calls were traced and that the trace span durations are greater
    than some minimum value.
    """

    python_spans = []
    with tracer.start_as_current_span("translate_submit_retrieve") as span:
        python_spans.append(span)
        translated = translate(native_bitflip_program, 1, quantum_processor_id)
        assert translated.program is not None
        if live_qpu_access:
            job_id = submit(translated.program, patch_values={}, quantum_processor_id=quantum_processor_id)
            retrieve_results(job_id, quantum_processor_id=quantum_processor_id)

    with tracer.start_as_current_span("translate_submit_retrieve_async") as span:
        python_spans.append(span)
        translated = await translate_async(native_bitflip_program, 1, quantum_processor_id)
        assert translated.program is not None
        if live_qpu_access:
            job_id = await submit_async(translated.program, patch_values={}, quantum_processor_id=quantum_processor_id)
            await retrieve_results_async(job_id, quantum_processor_id=quantum_processor_id)

    if live_qpu_access:
        with tracer.start_as_current_span("execute_on_qpu") as span:
            python_spans.append(span)
            executable = Executable(native_bitflip_program, shots=1)
            executable.execute_on_qpu(quantum_processor_id)

        with tracer.start_as_current_span("execute_on_qpu_async") as span:
            python_spans.append(span)
            executable = Executable(native_bitflip_program, shots=1)
            await executable.execute_on_qpu_async(quantum_processor_id)

    for span in python_spans:
        _verify_resource_spans(rust_global_trace_file, _TRANSLATE_QUIL_TO_ENCRYPTED_CONTROLLER_JOB, span)
        if live_qpu_access:
            _verify_resource_spans(rust_global_trace_file, _EXECUTE_CONTROLLER_JOB, span)
            _verify_resource_spans(rust_global_trace_file, _GET_CONTROLLER_JOB_RESULTS, span)


@pytest.fixture(scope="session")
def tracing_environment_variables() -> Generator[None, None, None]:
    """
    Sets environment variables to set the desired `EnvFilter` for the OTLP file export layer.
    """
    with mock.patch.dict(
        os.environ,
        {
            "RUST_LOG": "qcs=info",
            "QCS_API_TRACING_ENABLED": "1",
            "QCS_SDK_TESTS_KEEP_RUST_TRACES": "1",
            "QCS_API_PROPAGATE_OTEL_CONTEXT": "1",
        },
    ):
        yield


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


def _verify_resource_spans(
    rust_trace_file: str,
    span_name: str,
    parent_span: Span,
    count: int = 1,
    minimum_expected_duration: timedelta = _MIN_EXPECTED_GRPC_DURATION,
):
    """
    Open the Rust trace file and assert that it contains the expected number of
    spans specified by `span_name` under the given `parent_span`. Assert that the
    duration of these spans is greater than `minimum_expected_duration`.
    """
    with open(rust_trace_file) as f:
        resource_spans = []
        for line in f.readlines():
            resource_spans += json.loads(line)["resourceSpans"]

    parent_span_context = parent_span.get_span_context()
    if parent_span_context is None:
        raise RuntimeError("Parent span has no context")
    counter = Counter()
    for resource_span in resource_spans:
        for scope_span in resource_span["scopeSpans"]:
            for span in scope_span["spans"]:
                trace_id = int(span["traceId"], 16)
                if trace_id == parent_span_context.trace_id:
                    counter[span["name"]] += 1
                    if span["name"] == span_name:
                        duration_ns = int(span["endTimeUnixNano"]) - int(span["startTimeUnixNano"])
                        duration = timedelta(microseconds=duration_ns / 1000)
                        assert duration >= minimum_expected_duration

    assert (
        counter[span_name] == count
    ), f'Expected {count} spans with name {span_name} within parent "{parent_span.name}", but found {counter[span_name]}. See {rust_trace_file} trace {hex(parent_span_context.trace_id)}'


@pytest.fixture(scope="session")
def rust_global_trace_file() -> Generator[str, None, None]:
    """
    This session fixture is initialized only once across the test process. This is necessary
    because the Rust tracing subscriber is a global singleton that will panic if initialized
    more than once within a single process.

    The trace file is deleted after the test is run, unless the environment variable `QCS_SDK_TESTS_KEEP_RUST_TRACES` is set to `1`.
    """
    timestamp = datetime.now().astimezone(timezone.utc).isoformat()
    file_name = f"rust-traces-{timestamp}.json"
    try:
        os.makedirs(TRACE_DIR, exist_ok=True)
        rust_trace_file = os.path.join(TRACE_DIR, file_name)
        config = GlobalTracingConfig(
            export_process=SimpleConfig(
                subscriber=subscriber.Config(layer=layers.otel_otlp_file.Config(file_path=rust_trace_file))
            )
        )
        with Tracing(config=config):
            yield rust_trace_file
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
