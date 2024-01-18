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

from typing import final

from .. import layers

@final
class Config:
    """
    Configuration for the tracing subscriber. Currently, this only requires a single layer to be
    set on the `tracing_subscriber::Registry`.
    """

    def __new__(cls, *, layer: layers.Config) -> "Config": ...
