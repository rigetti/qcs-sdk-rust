from typing import Optional


class QcsClient:
    """
    Configuration for connecting and authenticating to QCS API resources.
    """

    def __new__(
        cls,
        tokens: Optional[QcsClientTokens] = None,
        api_url: Optional[str] = None,
        auth_server: Optional[QcsClientAuthServer] = None,
        grpc_api_url: Optional[str] = None,
        quilc_url: Optional[str] = None,
        qvm_url: Optional[str] = None,
    ) -> "QcsClient":
        """
        Construct a client from scratch.
        
        Use ``QcsClient.load`` to construct an environment-based profile.
        """
        ...
    
    @staticmethod
    def load(
        profile_name: Optional[str] = None,
        use_gateway: Optional[bool] = None,
    ) -> "QcsClient":
        """
        Load a QcsClient configuration using an environment-based configuration.

        See for details: https://docs.rigetti.com/qcs/references/qcs-client-configuration#environment-variables-and-configuration-files
        """
        ...


class QcsClientAuthServer:
    """Authentication server configuration for the QCS API."""

    @property
    def client_id(self) -> str: ...
    @client_id.setter
    def client_id(self, value: str): ...

    @property
    def issuer(self) -> str: ...
    @issuer.setter
    def issuer(self, value: str): ...


class QcsClientTokens:
    """Authentication tokens for the QCS API."""

    @property
    def bearer_access_token(self) -> Optional[str]: ...
    @bearer_access_token.setter
    def bearer_access_token(self, value: Optional[str]): ...

    @property
    def refresh_token(self) -> Optional[str]: ...
    @refresh_token.setter
    def refresh_token(self, value: Optional[str]): ...


class QcsGrpcClientError(RuntimeError):
    """Error encountered while loading a QCS gRPC API client."""


class QcsGrpcEndpointError(RuntimeError):
    """Error when trying to resolve the QCS gRPC API endpoint."""


class QcsGrpcError(RuntimeError):
    """Error during QCS gRPC API requests."""


class QcsLoadError(RuntimeError):
    """Error encountered while loading the QCS API client configuration."""
