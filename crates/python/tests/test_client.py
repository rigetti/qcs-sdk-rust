import pytest

from qcs_sdk import QcsClient
from qcs_sdk.qpu.client import QcsLoadError

@pytest.fixture
def default_client():
    return QcsClient()


@pytest.mark.asyncio
async def test_client_empty_profile_is_default(default_client: QcsClient):
    """The profile "empty" is configured to be similar to a default client."""
    client = await QcsClient.load(profile_name="empty")

    assert client == default_client


@pytest.mark.asyncio
async def test_client_default_profile_is_not_empty(default_client: QcsClient):
    """The "default" profile is configured to have a token, unlike the default client."""
    client = await QcsClient.load()

    assert client != default_client


@pytest.mark.asyncio
async def test_client_broken_raises():
    """Using a profile with broken configuration should surface the underlying error."""
    with pytest.raises(QcsLoadError, match=r"Expected auth server .* but it didn't exist"):
        await QcsClient.load(profile_name="broken")
