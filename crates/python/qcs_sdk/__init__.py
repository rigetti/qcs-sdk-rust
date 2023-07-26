# type: ignore
# See the following documentation for why this file is necessary:
# https://pyo3.rs/v0.18.0/python_typing_hints#__init__py-content

import sys

from .qcs_sdk import *


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


__doc__ = qcs_sdk.__doc__
__all__ = ["gather_diagnostics"] + getattr(qcs_sdk, "__all__", [])
