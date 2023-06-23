from qcs_sdk import reset_logging

def test_reset_logging():
    """
    Assert that resetting logging configuration does not panic.
    """
    reset_logging()

