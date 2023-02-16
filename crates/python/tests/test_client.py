import pytest
from urllib.parse import urlparse

from qcs_sdk import QcsClient
from qcs_sdk.qpu.client import QcsLoadError, QcsClientAuthServer, QcsClientTokens

@pytest.fixture
def default_client():
    return QcsClient()

def test_client_has_url_from_env(default_client: QcsClient):
    """The default client is configured with valid urls."""
    assert urlparse(default_client.api_url).geturl() != ""
    assert urlparse(default_client.grpc_api_url).geturl() != ""
    assert urlparse(default_client.quilc_url).geturl() != ""
    assert urlparse(default_client.qvm_url).geturl() != ""


def test_client_empty_profile_is_default(default_client: QcsClient):
    """The profile "empty" is configured to be similar to a default client."""
    client = QcsClient.load(profile_name="empty")

    assert client == default_client


def test_client_default_profile_is_not_empty(default_client: QcsClient):
    """The "default" profile is configured to have a token, unlike the default client."""
    client = QcsClient.load()

    assert client != default_client


def test_client_broken_raises():
    """Using a profile with broken configuration should surface the underlying error."""
    with pytest.raises(QcsLoadError, match=r"Expected auth server .* but it didn't exist"):
        QcsClient.load(profile_name="broken")


def test_client_auth_server_can_be_manually_defined():
    """Ensures that pyo3 usage is correct."""
    auth_server = QcsClientAuthServer(client_id="foo", issuer="bar")
    assert auth_server.client_id == "foo"
    assert auth_server.issuer == "bar"


def test_client_tokens_can_be_manually_defined():
    """Ensures that pyo3 usage is correct."""
    auth_server = QcsClientTokens(bearer_access_token="foo", refresh_token="bar")
    assert auth_server.bearer_access_token == "foo"
    assert auth_server.refresh_token == "bar"
