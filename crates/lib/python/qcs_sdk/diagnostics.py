import sys

from .qcs_sdk import __version__, _gather_diagnostics  # type: ignore


def get_report() -> str:
    """
    Return a string describing the package and its environment for use in bug reporting and diagnosis.

    Note: this format is not stable and its content may change between versions.
    """
    return f"""qcs-sdk-python version: {__version__}
Python version: {sys.version}
Python implementation: {sys.implementation.name}
Python implementation version: {sys.implementation.version.major}.{sys.implementation.version.minor}.{sys.implementation.version.micro}
Python C API version: {sys.api_version}
Python executable: {sys.executable}
venv prefix: {sys.prefix}
platform: {sys.platform}
{_gather_diagnostics()}
   """
