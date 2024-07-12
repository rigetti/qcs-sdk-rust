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
from qcs_sdk.qpu.api import retrieve_results, retrieve_results_async, submit, submit_async
from qcs_sdk.qpu.translation import (
    translate,
    translate_async,
)
from qcs_sdk._tracing_subscriber import GlobalTracingConfig, SimpleConfig, Tracing, subscriber, layers


DIR = os.path.dirname(os.path.abspath(__file__))
TRACE_DIR = os.path.join(DIR, "__output__", "traces")
os.makedirs(TRACE_DIR, exist_ok=True)


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_tracing_subscriber(
    tracing_environment_variables: None,
    native_bitflip_program: str,
    quantum_processor_id: str,
    tracer: trace.Tracer,
    rust_trace_file: str,
    live_qpu_access: bool,
):
    """
    Configure and initialize a Rust tracing subscriber for the QCS SDK. Run translation,
    job submission, and result retrieval using async and non-async methods. Assert the
    underlying gRPC calls were traced and that the trace span durations are greater
    than some minimum value.

    Note, this test uses `GlobalTracingConfig`, so no other test with such configuration
    can run within the same test process. Additional assertions on the collected traces
    may be added to this test. `CurrentThreadTracingConfig` is not viable for this test
    because we are testing `pyo3_asyncio` methods.

    By default, after the test is run, the Rust trace file is deleted. To keep the trace file
    for debugging, set the environment variable `QCS_SDK_TESTS_KEEP_RUST_TRACES` to `1`.
    """
    config = GlobalTracingConfig(export_process=SimpleConfig(subscriber=subscriber.Config(layer=layers.otel_otlp_file.Config(file_path=rust_trace_file))))

    spans = []
    with Tracing(config=config):
        with tracer.start_as_current_span("translate_submit_retrieve") as span:
            spans.append(span) 
            translated = translate(native_bitflip_program, 1, quantum_processor_id)
            assert translated.program is not None
            if live_qpu_access:
                job_id = submit(translated.program, patch_values={}, quantum_processor_id=quantum_processor_id)
                retrieve_results(job_id, quantum_processor_id=quantum_processor_id)

        with tracer.start_as_current_span("translate_submit_retrieve_async") as span:
            spans.append(span)
            translated = await translate_async(native_bitflip_program, 1, quantum_processor_id)
            assert translated.program is not None
            if live_qpu_access:
                job_id = await submit_async(translated.program, patch_values={}, quantum_processor_id=quantum_processor_id)
                await retrieve_results_async(job_id, quantum_processor_id=quantum_processor_id)

        if live_qpu_access:
            with tracer.start_as_current_span("execute_on_qpu") as span:
                spans.append(span)
                executable = Executable(native_bitflip_program, shots=1)
                executable.execute_on_qpu(quantum_processor_id)

            with tracer.start_as_current_span("execute_on_qpu_async") as span:
                spans.append(span)
                executable = Executable(native_bitflip_program, shots=1)
                await executable.execute_on_qpu_async(quantum_processor_id)

    for span in spans:
        _verify_resource_spans(rust_trace_file, _TRANSLATE_QUIL_TO_ENCRYPTED_CONTROLLER_JOB, span)
        if live_qpu_access:
            _verify_resource_spans(rust_trace_file, _EXECUTE_CONTROLLER_JOB, span)
            _verify_resource_spans(rust_trace_file, _GET_CONTROLLER_JOB_RESULTS, span)


@pytest.fixture(scope="session")
def tracing_environment_variables() -> Generator[None, None, None]:
    """
    Sets environment variables to set the desired `EnvFilter` for the OTLP file export layer.
    """
    with mock.patch.dict(
        os.environ,
        {
            "RUST_LOG": "qcs=info",
            "QCS_API_TRACING_ENABLED": "1"
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


def _verify_resource_spans(rust_trace_file: str, span_name: str, parent_span: Span, count: int = 1, minimum_expected_duration: timedelta = _MIN_EXPECTED_GRPC_DURATION):
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
                        duration_ns = span["endTimeUnixNano"] - span["startTimeUnixNano"]
                        duration = timedelta(microseconds=duration_ns / 1000)
                        assert duration >= minimum_expected_duration

    assert counter[span_name] == count, f"Expected {count} spans with name {span_name} within parent \"{parent_span.name}\", but found {counter[span_name]}. See {rust_trace_file} trace {hex(parent_span_context.trace_id)}"


@pytest.fixture(scope="function")
def rust_trace_file() -> Generator[str, None, None]:
    """
    Open a file for writing Rust traces. Delete the file after the test is run (unless
    `QCS_SDK_TESTS_KEEP_RUST_TRACES` is set to `1`).
    """
    timestamp = datetime.now().astimezone(timezone.utc).isoformat()
    file_name = f"rust-traces-{timestamp}.json"
    try:
        yield os.path.join(TRACE_DIR, file_name)
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
