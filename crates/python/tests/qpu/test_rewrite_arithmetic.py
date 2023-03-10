import pytest

from qcs_sdk.qpu.rewrite_arithmetic import (
    rewrite_arithmetic,
    build_patch_values,
    BuildPatchValuesError,
    RewriteArithmeticError,
)


def test_rewrite_arithmetic(
    bell_program: str,
):
    rewritten = rewrite_arithmetic(bell_program)
    assert rewritten.program.strip() == bell_program.strip()
    assert rewritten.recalculation_table == []


def test_rewrite_arithmetic_error():
    with pytest.raises(RewriteArithmeticError):
        rewrite_arithmetic("DECLARE --")


def test_build_patch_values():
    memory = { "a": [1.0], "b": [3] }
    output = build_patch_values(["a/b"], memory)
    assert output == { "__SUBST": [1/3], **memory }


def test_build_patch_values_error():
    with pytest.raises(BuildPatchValuesError):
        build_patch_values(["a/b"], {"a": [1], "b": [0]})
