from typing import Dict, Optional, final

from qcs_sdk.client import QCSClient

class GetQuiltCalibrationsError(RuntimeError):
    """An error occured while fetching Quil-T calibrations."""

    ...

class TranslationError(RuntimeError):
    """An error occured while translating a program."""

    ...

@final
class QuiltCalibrations:
    """Result of `get_quilt_calibrations`."""

    @property
    def quilt(self) -> str:
        """Calibrations suitable for use in a Quil-T program."""
        ...
    @quilt.setter
    def quilt(self, value: str): ...
    @property
    def settings_timestamp(self) -> Optional[str]:
        """ISO8601 timestamp of the settings used to generate these calibrations."""
        ...
    @settings_timestamp.setter
    def settings_timestamp(self, value: Optional[str]): ...

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

def get_quilt_calibrations(
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> QuiltCalibrations:
    """
    Retrieve the calibration data used for client-side Quil-T generation.

    :param quantum_processor_id: The ID of the quantum processor the job ran on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :returns QuiltCalibrations:

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises GetQuiltCalibrationsError: If there was a problem fetching Quil-T calibrations.
    """
    ...

async def get_quilt_calibrations_async(
    quantum_processor_id: str,
    client: Optional[QCSClient] = None,
    timeout: Optional[float] = None,
) -> QuiltCalibrations:
    """
    Retrieve the calibration data used for client-side Quil-T generation.
    (async analog of ``get_quilt_calibrations``)

    :param quantum_processor_id: The ID of the quantum processor the job ran on.
    :param client: The ``QCSClient`` to use. Creates one using environment configuration if unset - see https://docs.rigetti.com/qcs/references/qcs-client-configuration
    :param timeout: Maximum duration to wait for API calls to complete, in seconds.

    :returns QuiltCalibrations:

    :raises LoadClientError: If there is an issue loading the QCS Client configuration.
    :raises GetQuiltCalibrationsError: If there was a problem fetching Quil-T calibrations.
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

    def use_backend_v1(self) -> None:
        """
        Use the v1 backend for translation, available on QCS since 2018.
        """

    def use_backend_v2(self) -> None:
        """
        Use the v2 backend for translation, available on QCS since 2023.
        """
