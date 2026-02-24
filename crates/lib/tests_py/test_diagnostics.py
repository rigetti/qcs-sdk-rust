import asyncio

import pytest

from qcs_sdk import diagnostics

@pytest.mark.qcs_session
def test_get_report():
    """The async and sync reports are the same, and are generated without panicing."""

    async def async_report():
        return await diagnostics.get_report_async()

    assert diagnostics.get_report() == asyncio.run(async_report())
