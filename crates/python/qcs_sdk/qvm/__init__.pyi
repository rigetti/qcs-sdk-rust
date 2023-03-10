from typing import Dict, final

from .._register_data import RegisterData


@final
class QVMResultData:
    """
    Encapsulates data returned from the QVM after executing a program.
    """

    @staticmethod
    def from_memory_map(memory: Dict[str, RegisterData]) -> "QVMResultData":
        """
        Build a ``QVMResultData`` from a mapping of register names to a ``RegisterData`` matrix.
        """
        ...

    @property
    def memory(self) -> Dict[str, RegisterData]:
        """
        Get the mapping of register names (ie. "ro") to a ``RegisterData`` matrix containing the register values.
        """
        ...
