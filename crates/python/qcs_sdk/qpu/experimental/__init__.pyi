"""
This module supports experimental features that meet the following criteria:

* They support recent and compelling research in quantum computing.
* Implementation is specific to Rigetti's QPUs; implementation may
  not othewrise be expressed through a generalized quantum computing
  language, such as Quil or QASM.

As such, the features contained herein should be considered unstable, possibly
ephemeral, and subject to specific authorization checks.
"""

from qcs_sdk.qpu.experimental import (
    random as random,
)


__all__ = ["random"]
