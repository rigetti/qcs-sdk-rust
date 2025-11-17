
# The following code exposes the package contents under the target namespace.
from . import _qcs_sdk
assert isinstance(_qcs_sdk.__all__, list) and all(isinstance(s, str) for s in _qcs_sdk.__all__)
exec(f"from ._qcs_sdk import {', '.join(_qcs_sdk.__all__)}; __all__ = {_qcs_sdk.__all__}")
del _qcs_sdk
