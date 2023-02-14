"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""

from typing import Dict

from .._register_data import RegisterData

class QVMResultData:
    """
    Encapsulates data returned from the QVM after executing a program.
    """

    @staticmethod
    def from_memory_map(memory: Dict[str, RegisterData]) -> "QVMResultData": ...
    """
    Build a ``QVMResultData`` from a mapping of register names to a ``RegisterData`` matrix.
    """

    @property
    def memory(self) -> Dict[str, RegisterData]: ...
    """
    Get the mapping of register names (ie. "ro") to a ``RegisterData`` matrix containing the register values.
    """

    @memory.setter
    def memory(self, memory_map: Dict[str, RegisterData]): ...
    """
    Set the mapping of register names (ie. "ro") to a ``RegisterData`` matrix containing the register values.
    """
