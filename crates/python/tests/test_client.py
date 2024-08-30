import pytest
from urllib.parse import urlparse

from qcs_sdk.client import (
    QCSClient,
    LoadClientError,
    OAuthSession,
    RefreshToken,
    AuthServer,
)


@pytest.fixture
def default_client():
    return QCSClient()


def test_client_has_url_from_env(default_client: QCSClient):
    """The default client is configured with valid urls."""
    assert urlparse(default_client.api_url).geturl() != ""
    assert urlparse(default_client.grpc_api_url).geturl() != ""
    assert urlparse(default_client.quilc_url).geturl() != ""
    assert urlparse(default_client.qvm_url).geturl() != ""


@pytest.mark.not_qcs_session
@pytest.mark.asyncio
async def test_client_empty_profile_is_default(default_client: QCSClient):
    """The profile "empty" is configured to be similar to a default client."""
    client = QCSClient.load(profile_name="empty")

    assert client.api_url == default_client.api_url
    assert client.grpc_api_url == default_client.grpc_api_url
    assert client.quilc_url == default_client.quilc_url
    assert client.qvm_url == default_client.qvm_url


@pytest.mark.not_qcs_session
def test_client_default_profile_is_not_empty(default_client: QCSClient):
    """The "default" profile is configured to have a token, unlike the default client."""
    client = QCSClient.load()

    assert client != default_client


@pytest.mark.not_qcs_session
def test_client_broken_raises():
    """Using a profile with broken configuration should surface the underlying error."""
    with pytest.raises(
        LoadClientError, match=r"Expected auth server .* but it does not exist"
    ):
        QCSClient.load(profile_name="broken")


def test_client_oauth_session_can_be_manually_defined():
    """Ensures that pyo3 usage is correct."""
    auth_server = AuthServer("url", "issuer")
    session = OAuthSession(RefreshToken("refresh"), auth_server, "access")
    assert session.payload.refresh_token == "refresh"
    assert session.access_token == "access"
    assert session.auth_server == auth_server


def test_client_constructor():
    client = QCSClient(
        qvm_url="qvm_url",
        quilc_url="quilc_url",
        grpc_api_url="grpc_api_url",
        api_url="api_url",
    )
    assert client.qvm_url == "qvm_url"
    assert client.quilc_url == "quilc_url"
    assert client.grpc_api_url == "grpc_api_url"
    assert client.api_url == "api_url"
