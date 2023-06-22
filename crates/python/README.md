# QCS SDK Python

⚠️ In Development

`qcs-sdk-python` provides an interface to Rigetti [Quantum Cloud Services](https://docs.rigetti.com/qcs/) (QCS), allowing users
to compile and run Quil programs on Rigetti quantum processors. Internally, it is powered by the [QCS Rust SDK](https://github.com/rigetti/qcs-sdk-rust).

While this package can be used directly, [pyQuil](https://pypi.org/project/pyquil/) offers more functionality and a 
higher-level interface for building and executing Quil programs. This package is still in early development and breaking changes should be expected between minor versions.

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
