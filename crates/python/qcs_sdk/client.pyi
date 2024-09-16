from typing import Callable, Optional, final

class LoadClientError(RuntimeError):
    """Error encountered while loading the QCS API client configuration from the environment configuration."""

    ...

class BuildClientError(RuntimeError):
    """Error encountered while building the QCS API client configuration manually."""

    ...

@final
class QCSClient:
    """
    Configuration for connecting and authenticating to QCS APIs and resources.
    """

    def __new__(
        cls,
        oauth_session: Optional[OAuthSession] = None,
        api_url: Optional[str] = None,
        grpc_api_url: Optional[str] = None,
        quilc_url: Optional[str] = None,
        qvm_url: Optional[str] = None,
    ) -> "QCSClient":
        """
        Manually construct a `QCSClient`.

        Prefer to use `QCSClient.load` to construct an environment-based profile.
        """
        ...
    @staticmethod
    def load(
        profile_name: Optional[str] = None,
    ) -> "QCSClient":
        """
        Create a `QCSClient` configuration using an environment-based configuration.

        :param profile_name: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.

        :raises `LoadClientError`: If there is an issue loading the profile defails from the environment.

        See the [QCS documentation](https://docs.rigetti.com/qcs/references/qcs-client-configuration#environment-variables-and-configuration-files)
        for more details.
        """
        ...
    @property
    def api_url(self) -> str:
        """URL to access the QCS API."""
        ...
    @property
    def grpc_api_url(self) -> str:
        """URL to access the QCS gRPC API."""
        ...
    @property
    def quilc_url(self) -> str:
        """URL to access the ``quilc`1 compiler."""
        ...
    @property
    def qvm_url(self) -> str:
        """URL to access the QVM."""
        ...
    @property
    def oauth_session(self) -> OAuthSession:
        """Get a copy of the OAuth session."""

@final
class OAuthSession:
    def __new__(
        cls,
        grant_payload: RefreshToken | ClientCredentials | ExternallyManaged,
        auth_server: AuthServer,
        access_token: str | None = None,
    ) -> OAuthSession: ...
    @property
    def access_token(self) -> str:
        """Get the current access token.

        This is an unvalidated copy of the access token. Meaning it can become stale, or may already be stale. See the `validate` `request_access_token` and methods.
        """

    @property
    def auth_server(self) -> AuthServer:
        """The auth server."""

    @property
    def payload(self) -> RefreshToken | ClientCredentials:
        """Get the payload used to request an access token."""

    def request_access_token(self) -> str:
        """Request a new access token."""

    async def request_access_token_async(self) -> str:
        """Request a new access token."""

    def validate(self) -> str:
        """Validate the current access token, returning it if it is valid.

        If the token is invalid, a `ValueError` will be raised with a description of why the token failed validation.
        """

@final
class AuthServer:
    def __new__(cls, client_id: str, issuer: str) -> AuthServer: ...
    @staticmethod
    def default() -> AuthServer:
        """Get the default Okta auth server."""

    @property
    def client_id(self) -> str:
        """The client's Okta ID."""

    @property
    def issuer(self) -> str:
        """The Okta issuer URL."""

@final
class RefreshToken:
    def __new__(cls, refresh_token: str) -> RefreshToken: ...
    @property
    def refresh_token(self) -> str:
        """The refresh token."""
    @refresh_token.setter
    def refresh_token(self, refresh_token: str):
        """Set the refresh token."""

@final
class ClientCredentials:
    def __new__(cls, client_id: str, client_secret: str) -> ClientCredentials: ...
    @property
    def client_id(self) -> str:
        """The client ID."""
    @property
    def client_secret(self) -> str:
        """The client secret."""

@final
class ExternallyManaged:
    def __new__(
        cls, refresh_function: Callable[[AuthServer], str]
    ) -> ExternallyManaged:
        """Manages access tokens by utilizing a user-provided refresh function.

        The refresh function should return a valid access token, or raise an exception if it cannot.

        .. testcode::
            from qcs_apiclient_common.configuration import AuthServer, ExternallyManaged, OAuthSession

            def refresh_function(auth_server: AuthServer) -> str:
                return "my_access_token"

            externally_managed = ExternallyManaged(refresh_function)
            session = OAuthSession(externally_managed, AuthServer.default())
        """
