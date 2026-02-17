"""An interface to Rigetti [Quantum Cloud Services](https://docs.rigetti.com/qcs/) (QCS).

These APIs allow users to compile and run Quil programs on Rigetti quantum processors.
Internally, it is powered by the [QCS Rust SDK](https://github.com/rigetti/qcs-sdk-rust).

This package is still in development and breaking changes should be expected between minor versions.
"""

from . import _qcs_sdk
from .client import QCSClient  # noqa

_additional_exports = [
    "QCSClient",
]

# The following code exposes the package contents under the same namespace without the `_` prefix.
assert isinstance(_qcs_sdk.__all__, list) and all(isinstance(s, str) for s in _qcs_sdk.__all__)
exec(
    f"from ._qcs_sdk import {', '.join(_qcs_sdk.__all__)}; "
    f"__all__ = {_qcs_sdk.__all__ + _additional_exports}"
)
del _qcs_sdk

