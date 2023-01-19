from enum import Enum

from .api import *
from .qpu.client import QcsClient as QcsClient

class QcsExecutionError(RuntimeError):
    """Error during QCS program execution."""
    ...


class Service(Enum):
    Quilc = "Quilc",
    Qvm = "Qvm",
    Qcs = "Qcs",
    Qpu = "Qpu",
