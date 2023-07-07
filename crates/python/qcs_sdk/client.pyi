from typing import Optional, final

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
        tokens: Optional[QCSClientTokens] = None,
        api_url: Optional[str] = None,
        auth_server: Optional[QCSClientAuthServer] = None,
        grpc_api_url: Optional[str] = None,
        quilc_url: Optional[str] = None,
        qvm_url: Optional[str] = None,
    ) -> "QCSClient":
        """
        Manually construct a ``QCSClient``.

        Prefer to use ``QCSClient.load`` to construct an environment-based profile.
        """
        ...
    @staticmethod
    def load(
        profile_name: Optional[str] = None,
    ) -> "QCSClient":
        """
        Create a ``QCSClient`` configuration using an environment-based configuration.

        :param profile_name: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.

        :raises LoadClientError: If there is an issue loading the profile defails from the environment.

        See for details: https://docs.rigetti.com/qcs/references/qcs-client-configuration#environment-variables-and-configuration-files
        """
        ...
    @staticmethod
    async def load_async(
        profile_name: Optional[str] = None,
    ) -> "QCSClient":
        """
        Create a ``QCSClient`` configuration using an environment-based configuration.
        (async analog of ``QCSClient.load``)

        :param profile_name: The QCS setting's profile name to use. If ``None``, the default value configured in your environment is used.

        :raises LoadClientError: If there is an issue loading the profile defails from the environment.

        See for details: https://docs.rigetti.com/qcs/references/qcs-client-configuration#environment-variables-and-configuration-files
        """
        ...
    @property
    def api_url(self) -> str:
        """URL to access the QCS API."""
        ...
    @property
    def grpc_api_url(self) -> str:
        """URL to access the gRPC API."""
        ...
    @property
    def quilc_url(self) -> str:
        """URL to access the `quilc` compiler."""
        ...
    @property
    def qvm_url(self) -> str:
        """URL to access the QVM."""
        ...

@final
class QCSClientAuthServer:
    """Authentication server configuration for the QCS API."""

    def __new__(cls, client_id: str, issuer: str) -> "QCSClientAuthServer":
        """
        Manually define authentication server parameters.

        :param client_id: The OAuth application client ID. If ``None``, a default value is used.
        :param issuer: The OAuth token issuer url. If ``None``, a default value is used.
        """
        ...
    @property
    def client_id(self) -> str: ...
    @client_id.setter
    def client_id(self, value: str): ...
    @property
    def issuer(self) -> str: ...
    @issuer.setter
    def issuer(self, value: str): ...

@final
class QCSClientTokens:
    """Authentication tokens for the QCS API."""

    def __new__(
        cls,
        bearer_access_token: str,
        refresh_token: str,
    ) -> "QCSClientTokens":
        """
        Manually define authentication session tokens.

        :param bearer_access_token: The session token from an OAuth issuer.
        :param refresh_token: A credential to refresh the bearer_access_token when it expires.
        """
        ...
    @property
    def bearer_access_token(self) -> Optional[str]: ...
    @bearer_access_token.setter
    def bearer_access_token(self, value: Optional[str]): ...
    @property
    def refresh_token(self) -> Optional[str]: ...
    @refresh_token.setter
    def refresh_token(self, value: Optional[str]): ...
