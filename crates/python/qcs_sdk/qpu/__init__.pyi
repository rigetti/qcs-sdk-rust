from typing import List, Optional

from .client import QCSClient
from ._result_data import (
    QPUResultData as QPUResultData,
    ReadoutValues as ReadoutValues,
)


class ListQuantumProcessorsError(RuntimeError):
    """A request to list available Quantum Processors failed."""
    ...


def list_quantum_processors(
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> List[str]:
    """
    Returns all available Quantum Processor (QPU) IDs.

    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :raises ListQuantumProcessorsError: If the request to list available QPU IDs failed.
    """
    ...


async def list_quantum_processors_async(
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> List[str]:
    """
    Returns all available Quantum Processor IDs.
    (async analog of ``list_quantum_processors``)

    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :raises ListQuantumProcessorsError: If the request to list available QPU IDs failed.
    """
    ...