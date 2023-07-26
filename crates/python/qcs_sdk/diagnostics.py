import sys

from .qcs_sdk import __version__, _gather_diagnostics


def gather_diagnostics() -> str:
    return f"""qcs-sdk-python version: {__version__}
Python version: {sys.version}
Python implementation: {sys.implementation.name}
Python implementation version: {sys.implementation.version.major}.{sys.implementation.version.minor}.{sys.implementation.version.micro}
Python C api version: {sys.api_version}
Python executable: {sys.executable}
venv prefix: {sys.prefix}
platform: {sys.platform}
{_gather_diagnostics()}
    """
