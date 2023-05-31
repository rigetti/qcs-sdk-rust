from typing import Dict, List, Sequence, Union, Mapping, final

class RewriteArithmeticError(RuntimeError):
    """
    Errors that can result from rewriting arithmetic:

    - The Quil program could not be parsed.
    - Parametric arithmetic in the Quil program could not be rewritten.
    """

    ...

class BuildPatchValuesError(RuntimeError):
    """
    Errors that can result from building patch values:

    - Failed to interpret the recalculation values.
    - Failed to build patch values.
    """

    ...

@final
class RewriteArithmeticResult:
    """
    Result of a ``rewrite_arithmetic call``.

    Provides the information necessary to later patch-in memory values to a compiled program.
    """

    @property
    def program(self) -> str:
        """The rewritten program."""
        ...
    @property
    def recalculation_table(self) -> List[str]:
        """
        The expressions used to fill-in the `__SUBST` memory location.

        The expression index in this vec is the same as that in `__SUBST`.
        """
        ...

def build_patch_values(
    recalculation_table: Sequence[str],
    memory: Mapping[str, Union[Sequence[float], Sequence[int]]],
) -> Dict[str, List[float]]:
    """
    Evaluate the expressions in `recalculation_table` using the numeric values provided in `memory`.

    :raises BuildPatchValuesError: If patch values could not be built.
    """
    ...

def rewrite_arithmetic(native_quil: str) -> RewriteArithmeticResult:
    """
    Rewrite parametric arithmetic such that all gate parameters are only memory
    references to newly declared memory location (`__SUBST`).

    A "recalculation" table is provided which can be used to populate the memory
    when needed (see ``build_patch_values``).

    :raises RewriteArithmeticError: If the program fails to parse, or parameter arithmetic cannot be rewritten.
    """
    ...
