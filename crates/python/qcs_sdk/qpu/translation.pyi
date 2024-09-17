from typing import Dict, Optional, final
from enum import Enum, auto

from qcs_sdk.client import QCSClient

class TranslationError(RuntimeError):
    """Errors that can occur while using the translation module."""

    ...

@final
class TranslationResult:
    """
    The result of a call to `translate` which provides information about the translated program.
    """

    @property
    def program(self) -> str:
        """The translated programs."""
        ...
    @property
    def ro_sources(self) -> Optional[Dict[str, str]]:
        """A mapping from the program's memory references to the key used to index the results map."""
        ...

@final
class TranslationBackend(Enum):
    V1 = auto()
    V2 = auto()

def get_quilt_calibrations(
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> str:
    """
    Retrieve the calibration data used for client-side Quil-T generation.

    :param quantum_processor_id: The ID of the quantum processor the job ran on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :returns str: The Quil calibration program for the requested quantum processor.

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises TranslationError: If there was a problem fetching Quil-T calibrations.
    """
    ...

async def get_quilt_calibrations_async(
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> str:
    """
    Retrieve the calibration data used for client-side Quil-T generation.
    (async analog of ``get_quilt_calibrations``)

    :param quantum_processor_id: The ID of the quantum processor the job ran on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :returns str: The Quil calibration program for the requested quantum processor.

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises TranslationError: If there was a problem fetching Quil-T calibrations.
    """
    ...

def translate(
    native_quil: str,
    num_shots: int,
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
    translation_options: Optional[TranslationOptions] = None,
) -> TranslationResult:
    """
    Translates a native Quil program into an executable program.

    :param native_quil: A Quil program.
    :param num_shots: The number of shots to perform.
    :param quantum_processor_id: The ID of the quantum processor the executable will run on (e.g. "Aspen-M-2").
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :returns TranslationResult:

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises TranslationError: If the `native_quil` program could not be translated.
    """
    ...

async def translate_async(
    native_quil: str,
    num_shots: int,
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
    translation_options: Optional[TranslationOptions] = None,
) -> TranslationResult:
    """
    Translates a native Quil program into an executable program.
    (async analog of ``translate``)

    :param native_quil: A Quil program.
    :param num_shots: The number of shots to perform.
    :param quantum_processor_id: The ID of the quantum processor the executable will run on (e.g. "Aspen-M-2").
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration

    :returns TranslationResult:

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises TranslationError: If the `native_quil` program could not be translated.
    """
    ...

@final
class TranslationOptions:
    """
    Options for translating via the QCS API.
    """

    @property
    def backend(self) -> Optional[TranslationBackend]:
        """
        Get the selected translation backend
        """
    def use_backend_v1(self) -> None:
        """
        Use the v1 backend for translation, available on QCS since 2018.
        """
    def use_backend_v2(self) -> None:
        """
        Use the v2 backend for translation, available on QCS since 2023.
        """
    @staticmethod
    def v1() -> "TranslationOptions":
        """
        Use the v1 backend for translation, available on QCS since 2018.
        """
    @staticmethod
    def v2(
        *,
        prepend_default_calibrations: Optional[bool] = None,
        passive_reset_delay_seconds: Optional[float] = None,
        allow_unchecked_pointer_arithmetic: Optional[bool] = None,
        allow_frame_redefinition: Optional[bool] = None,
    ) -> "TranslationOptions":
        """
        Use the v2 backend for translation, available on QCS since 2023.

        :param: prepend_default_calibrations: If False, do not prepend the default calibrations to the translated
        program.
        :param: passive_reset_delay_seconds: The delay between passive resets, in seconds.
        :param: allow_unchecked_pointer_arithmetic: If True, disable runtime memory bounds checking. Only available to
        certain users.
        :param: allow_frame_redefinition: If True, allow defined frames to differ from Rigetti defaults. Only available to certain users.
        Otherwise, only ``INITIAL-FREQUENCY`` and ``CHANNEL-DELAY`` may be modified.
        """
    def encode_as_protobuf(self) -> bytes:
        """
        Serialize these translation options into the Protocol Buffer format.
        """
