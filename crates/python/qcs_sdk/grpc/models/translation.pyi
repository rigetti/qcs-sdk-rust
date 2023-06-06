from typing import Optional, final

class TranslationOptions:
    translation_backend: Optional[TranslationBackend] = None

    ...

@final
class TranslationBackend:
    """
    Object specifying which translation backend to use to translate a particular program,
    and for that given backend, what options to apply.

    Variants:
        ``v1``: Corresponds to the V1 translation backend.
        ``v2``: Corresponds to the V2 translation backend.

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlying values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """

    def is_v1(self) -> bool: ...
    def is_v2(self) -> bool: ...
    def as_v1(self) -> Optional[BackendV1Options]: ...
    def as_v2(self) -> Optional[BackendV2Options]: ...
    def to_v1(self) -> BackendV1Options: ...
    def to_v2(self) -> BackendV2Options: ...
    @staticmethod
    def from_v1(inner: BackendV1Options) -> "TranslationBackend": ...
    @staticmethod
    def from_v2(inner: BackendV2Options) -> "TranslationBackend": ...

class BackendV1Options:
    """
    Options for the V1 translation backend.
    """

class BackendV2Options:
    """
    Options for the V2 translation backend.
    """