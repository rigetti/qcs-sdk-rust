from qcs_sdk.diagnostics import get_report


def test_get_report():
    """
    Assert that gathering diagnostics doesn't panic.
    """
    get_report()
