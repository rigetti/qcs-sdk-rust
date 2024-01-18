from collections import Counter
from datetime import timedelta
import json
import os
from uuid import uuid4
from typing import Generator 
import pytest

from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import (
    BatchSpanProcessor,
    ConsoleSpanExporter,
)


from qcs_sdk.qpu.translation import (
    translate,
    translate_async,
)
from qcs_sdk._tracing_subscriber import GlobalTracingConfig, SimpleConfig, Tracing, subscriber, layers


DIR = os.path.dirname(os.path.abspath(__file__))
TRACE_DIR = os.path.join(DIR, "_fixtures", "traces")


@pytest.mark.qcs_session
@pytest.mark.asyncio
async def test_translate(
    native_bitflip_program: str,
    quantum_processor_id: str,
    tracer: trace.Tracer,
    rust_trace_file: str,
):
    """
    Configure and initialize a Rust tracing subscriber for the QCS SDK. Run a translation
    job against the configured service (see `qcs_session`). Assert that it was traced.

    Note, this test uses `GlobalTracingConfig`, so no other test with such configuration
    can run within the same test process. Additional assertions on the collected traces
    may be added to this test.

    By default, after the test is run, the Rust trace file is deleted. To keep the trace file
    for debugging, set the environment variable `QCS_SDK_TESTS_KEEP_RUST_TRACES` to `1`.
    """
    config = GlobalTracingConfig(export_process=SimpleConfig(subscriber=subscriber.Config(layer=layers.otel_otlp_file.Config(file_path=rust_trace_file))))
    async with Tracing(config=config):
        with tracer.start_as_current_span("test_translate") as span:
            translated = translate(native_bitflip_program, 1, quantum_processor_id)
            assert translated.program
            translated = await translate_async(native_bitflip_program, 1, quantum_processor_id)
            assert translated.program
            trace_id = span.get_span_context().trace_id

    _assert_translation_resource_spans(rust_trace_file, trace_id, 2)    



_TRANSLATION_REQUEST_SPAN_NAME = "/services.translation.Translation/TranslateQuilToEncryptedControllerJob"
_MIN_EXPECTED_TRANSLATION_DURATION = timedelta(milliseconds=10)
"""
The minimum amount of time we would expect a translation call to take. Note, this is a fairly
low duration, but it is sufficient to presume the network call was properly traced and that
the translation span is not empty.
"""


def _assert_translation_resource_spans(rust_trace_file: str, expected_trace_id: int, translation_count: int):
    """
    Open the Rust trace file and assert that it contains the expected number of translation
    spans with the expected duration.
    """
    with open(os.path.join(TRACE_DIR, rust_trace_file)) as f:
        resource_spans = []
        for line in f.readlines():
            resource_spans += json.loads(line)["resourceSpans"]

    counter = Counter()
    for resource_span in resource_spans:
        for scope_span in resource_span["scopeSpans"]:
            for span in scope_span["spans"]:
                trace_id = int(span["traceId"], 16)
                assert trace_id == expected_trace_id
                counter[span["name"]] += 1
                if span["name"] == _TRANSLATION_REQUEST_SPAN_NAME:
                    duration_ns = span["endTimeUnixNano"] - span["startTimeUnixNano"]
                    duration = timedelta(microseconds=duration_ns / 1000)
                    assert duration >= _MIN_EXPECTED_TRANSLATION_DURATION

    assert counter[_TRANSLATION_REQUEST_SPAN_NAME] == translation_count


@pytest.fixture(scope="function")
def rust_trace_file() -> Generator[str, None, None]:
    """
    Open a file for writing Rust traces. Delete the file after the test is run (unless
    `QCS_SDK_TESTS_KEEP_RUST_TRACES` is set to `1`).
    """
    file_name = f"rs-{uuid4()}.json"
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
