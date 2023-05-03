from typing import Dict, List, Optional
from ..qvm import QVMResultData
from ..qpu.client import QCSClient

def run(
    quil: str,
    shots: int,
    readouts: List[str],
    params: Dict[str, List[float]],
    client: Optional[QCSClient] = None,
) -> QVMResultData: ...

"""
    Runs the given program on the QVM.

    :param quil: A quil program as a string.
    :param shots: The number of times to run the program. Should be a value greater than zero.
    :param readouts: A list of memory region names to get back from the QVM at the end of execution.
    :param params: A mapping of memory region names to their desired values.
    :param client: An optional ``QCSClient`` to use. If unset, creates one using the environemnt configuration (see https://docs.rigetti.com/qcs/references/qcs-client-configuration).

    :returns: A ``QVMResultData`` containing the final state of of memory for the requested readouts after the program finished running.

    :raises QVMError: If one of the parameters is invalid, or if there was a problem communicating with the QVM server.
"""

async def run_async(
    quil: str,
    shots: int,
    readouts: str,
    params: Dict[str, List[float]],
    client: Optional[QCSClient] = None,
) -> QVMResultData: ...
def get_version_info(client: Optional[QCSClient] = None) -> str: ...
async def get_version_info_async(client: Optional[QCSClient] = None) -> str: ...
