from qcs_sdk import get_diagnostics_report


def test_gather_diagnostics():
    """
    Assert that gathering diagnostics doesn't panic.
    """
    gather_diagnostics()
