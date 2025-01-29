# *****************************************************************************
# *                             AUTO-GENERATED CODE                           *
# *                                                                           *
# * This code was generated by the `pyo3-tracing-subscriber` crate. Any       *
# * modifications to this file should be made to the script or the generation *
# * process that produced this code. Specifically, see:                       *
# * `pyo3_tracing_subscriber::stubs::write_stub_files`                        *
# *                                                                           *
# * Do not manually edit this file, as your changes may be overwritten the    *
# * next time the code is generated.                                          *
# *****************************************************************************

from typing import Dict, Optional, final


__all__ = [
    'InstrumentationLibrary',
]


@final
class InstrumentationLibrary:
    """
    Information about a library or crate providing instrumentation.
    
    An instrumentation library should be named to follow any naming conventions
    of the instrumented library (e.g. 'middleware' for a web framework).
    
    See the `instrumentation libraries <https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/overview.md#instrumentation-libraries>`_
    spec for more information.
    """

    def __new__(
        cls,
        name: str,
        version: Optional[str] = None,
        schema_url: Optional[str] = None,
        attributes: Optional[Dict[str, str]] = None,
    ) -> "InstrumentationLibrary":
        """
        Initializes a new instance of `InstrumentationLibrary`.

        :param name: The name of the instrumentation library.
        :param version: The version of the instrumentation library.
        :param schema_url: The `schema URL <https://opentelemetry.io/docs/specs/otel/schemas/>`_ of
            the instrumentation library.
        :param attributes: The attributes of the instrumentation library.
        """
        ...

