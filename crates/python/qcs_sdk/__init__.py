# type: ignore
# See the following documentation for why this file is necessary:
# https://pyo3.rs/v0.18.0/python_typing_hints#__init__py-content

from typing import Type
from .qcs_sdk import *
from . import _unitary_set

from types import ModuleType


def _monkey_patch(module: ModuleType, attribute: Type):
    setattr(module, attribute.__name__, attribute)
    if hasattr(module, "__all__"):
        module.__all__.append(attribute.__name__)


_monkey_patch(qcs_sdk.qpu.experimental.randomized_measurements, _unitary_set.UnitarySet)


__doc__ = qcs_sdk.__doc__
__all__ = getattr(qcs_sdk, "__all__", []) + ["diagnostics"]
