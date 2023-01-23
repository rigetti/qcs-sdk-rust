from typing import Any, Dict, Optional

DEFAULT_COMPILER_TIMEOUT: int


class QuilcError(RuntimeError):
    ...


class CompilerOpts:
    """A set of options that determine the behavior of compiling programs with quilc."""

    timeout: Optional[int] 
    """The number of seconds to wait before timing out. If `None`, there is no timeout."""

    def __new__(
        timeout: Optional[int] = DEFAULT_COMPILER_TIMEOUT
    ) -> "CompilerOpts":
        ...

    @staticmethod
    def default() -> "CompilerOpts": ...


class TargetDevice:
    isa: Any
    specs: Dict[str, str]
