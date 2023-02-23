from typing import Optional

DEFAULT_COMPILER_TIMEOUT: int
"""Number of seconds to wait before timing out."""

class QuilcError(RuntimeError): ...

class CompilerOpts:
    """A set of options that determine the behavior of compiling programs with quilc."""

    @property
    def timeout(self) -> Optional[float]:
        """The number of seconds to wait before timing out. If `None`, there is no timeout."""
        ...
    def __new__(
        cls, timeout: Optional[float] = DEFAULT_COMPILER_TIMEOUT
    ) -> "CompilerOpts": ...
    @staticmethod
    def default() -> "CompilerOpts": ...

class TargetDevice: ...
