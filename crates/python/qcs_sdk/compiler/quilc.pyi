from typing import Optional, final

from ..qpu.isa import InstructionSetArchitecture
from ..qpu.client import QCSClient

DEFAULT_COMPILER_TIMEOUT: float
"""Number of seconds to wait before timing out."""


class QuilcError(RuntimeError):
    """
    A number of errors that can occur when compiling with Quilc:
    - The ``InstructionSetArchitecture`` could not be converted into the Quilc format.
    - Connecting to the Quilc host failed.
    - Quilc returned an error during compilation.
    - The program was not parsed correctly.
    """
    ...


@final
class CompilerOpts:
    """A set of options that determine the behavior of compiling programs with quilc."""

    def __new__(
        cls,
        /,
        timeout: Optional[float] = DEFAULT_COMPILER_TIMEOUT,
        protoquil: Optional[bool] = None,
    ) -> "CompilerOpts": ...

    @staticmethod
    def default() -> "CompilerOpts": ...


@final
class TargetDevice:
    """
    Architectural description of device to compile for.
    """

    @staticmethod
    def from_isa(
        isa: InstructionSetArchitecture,
    ) -> "TargetDevice":
        """
        Create a ``TargetDevice`` based on an ``InstructionSetArchitecture``.
        
        :param isa: ``InstructionSetArchitecture`` that describes the target device.

        :raises QuilcError: If the ``InstructionSetArchitecture`` cannot be converted
        into a format that Quilc understands.
        """
        ...

    @staticmethod
    def from_json(
        value: str,
    ) -> "TargetDevice":
        """
        Create a ``TargetDevice`` based on its JSON representation.

        :param value: The JSON representation of a ``TargetDevice`` that describes the target device.

        :raises ValueError: If the JSON is malformed.
        """
        ...


def compile_program(
    quil: str,
    target: TargetDevice,
    client: Optional[QCSClient] = ...,
    options: Optional[CompilerOpts] = ...,
) -> str:
    """
    Compile a quil program for a target device.

    :param quil: The Quil program to compile.
    :param target: Architectural description of device to compile for.
    :param client: Optional client configuration. If ``None``, a default one is created.
    :param options: Optional compiler options. If ``None``, default values are used.

    :raises QuilcError: If compilation fails.
    """
    ...


async def compile_program_async(
    quil: str,
    target: TargetDevice,
    client: Optional[QCSClient] = ...,
    options: Optional[CompilerOpts] = ...,
) -> str:
    """
    Compile a quil program for a target device.
    (async analog of ``compile_program``)

    :param quil: The Quil program to compile.
    :param target: Architectural description of device to compile for.
    :param client: Optional client configuration. If ``None``, a default one is created.
    :param options: Optional compiler options. If ``None``, default values are used.

    :raises QuilcError: If compilation fails.
    """
    ...


def get_version_info(
    client: Optional[QCSClient] = ...,
) -> str:
    """
    Fetch the version information from the running Quilc service.

    :param client: Optional client configuration. If ``None``, a default one is created.

    :raises QuilcError: If the is a failure connecting to Quilc.
    """
    ...


async def get_version_info_async(
    client: Optional[QCSClient] = ...,
) -> str:
    """
    Fetch the version information from the running Quilc service.
    (async analog of ``get_version_info``)

    :param client: Optional client configuration. If ``None``, a default one is created.

    :raises QuilcError: If the is a failure connecting to Quilc.
    """
    ...
