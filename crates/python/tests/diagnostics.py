from qcs_sdk import gather_diagnostics


def test_gather_diagnostics():
    """
    Assert that gathering diagnostics doesn't panic.
    """
    gather_diagnostics()
