# QCS SDK Python

⚠️ In Development

`qcs-sdk-python` provides an interface to Rigetti [Quantum Cloud Services](https://docs.rigetti.com/qcs/) (QCS), allowing users
to compile and run Quil programs on Rigetti quantum processors. Internally, it is powered by the [QCS Rust SDK](https://github.com/rigetti/qcs-sdk-rust).

While this package can be used directly, [pyQuil](https://pypi.org/project/pyquil/) offers more functionality and a 
higher-level interface for building and executing Quil programs. This package is still in early development and breaking changes should be expected between minor versions.

# Documentation

Documentation for the current release of `qcs_sdk` is published [here](https://rigetti.github.io/qcs-sdk-rust/qcs_sdk.html). Every version of `qcs_sdk` ships [with type stubs](https://github.com/rigetti/qcs-sdk-rust/tree/main/crates/python/qcs_sdk) that can provide type hints and documentation to Python tooling and editors.

## Troubleshooting

### Enabling Debug logging

This package integrates with Python's [logging facility](https://docs.python.org/3/library/logging.html) through a Rust crate called [`pyo3_log`](https://docs.rs/pyo3-log/latest/pyo3_log/). The quickest way to get started is to just enable debug logging:

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

Because this is implemented with Rust, there are some important differences in regards to log levels and filtering.

#### The `TRACE` log level

Rust has a `TRACE` log level that doesn't exist in Python. It is less severe than `DEBUG` and is set to a value of 5. While the `DEBUG` level is recommended for troubleshooting, you can choose to target `TRACE` level logs and below like so:

```python
import logging
logging.basicConfig(level=5)
```

#### Runtime Configuration and Caching
 
`pyo3_log` caches loggers and their level filters to improve performance. This means that logger re-configuration done at runtime may cause unexpected logging behavior in certain situations. If this is a concern, [this section of the pyo3_log documentation](https://docs.rs/pyo3-log/latest/pyo3_log/#performance-filtering-and-caching) goes into more detail.

These caches can be reset using the following:

```python
qcs_sdk.reset_logging()
```

This will allow the logging handlers to pick up the most recently-applied configuration from the Python side.

#### Filtering Logs

Because the logs are emitted from a Rust library, the logger names will correspond to the fully qualified path of the Rust module in the library where the log occurred. These fully qualified paths all have their own logger, and have to be configured individually.

For example, say you wanted to disable the following log:

```
DEBUG:hyper.proto.h1.io:flushed 124 bytes
```

You could get the logger for `hyper.proto.h1.io` and disable it like so:

```python
logging.getLogger("hyper.proto.h1.io").disabled = True
```

This can become cumbersome, since there are a handful of libraries all logging from a handful of modules that you may not be concerned with. A less cumbersome, but more heavy handed approach is to apply a filter to all logging handlers at runtime. For example, if you only cared about logs from a `qcs` library, you could setup a log filter like so:

```python
class QCSLogFilter(logging.Filter):
    def filter(self, record) -> bool:
        return "qcs" in record.name

for handler in logging.root.handlers:
    handler.addFilter(QCSLogFilter())
```

This applies to all logs, so you may want to tune the `filter` method to include other logs you care about. See the caching section above for important information about the application of these filters.

#### OpenTelemetry Integration

This package supports collection of [OpenTelemetry trace data](https://opentelemetry.io/docs/concepts/signals/traces/). Clients may configure any OpenTelemetry [collector](https://opentelemetry.io/docs/collector/) that supports the [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/). _Rigetti will not have access to the OpenTelemetry data you collect_.

To enable the integration, you should install the `qcs-sdk[tracing-opentelemetry]` extra; this installs [opentelemetry-api](https://pypi.org/project/opentelemetry-api/). By default, no tracing data is collected at runtime. Because the QCS-SDK is built as a pyo3 Rust extension-module, you will need to use [pyo3-tracing-subscriber](https://crates.io/crates/pyo3-tracing-subscriber) to configure collection of your client network requests. See `qcs_sdk._tracing_subscriber` module level documentation for more detail.

```python
import my_module
from qcs_sdk._tracing_subscriber import (
    GlobalTracingConfig,
    SimpleConfig,
    Tracing,
    subscriber,
)
from qcs_sdk._tracing_subscriber.layers import otel_otlp


def main():
    tracing_configuration = GlobalTracingConfig(
        export_process=SimpleConfig(
            subscriber=subscriber.Config(
                # By default this supports the standard OTLP environment variables.
                # See https://opentelemetry.io/docs/specs/otel/protocol/exporter/
                layer=otel_otlp.Config()
            )
        )
    )
    with Tracing(config=config):
        result = my_module.example_function()
        my_module.other_example_function(result)

if __name__ == '__main__':
    main()
```
