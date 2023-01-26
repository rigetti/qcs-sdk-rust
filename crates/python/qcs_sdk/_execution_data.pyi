"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""

import datetime
from typing import List, Optional

class _IntegerReadoutValues:
    @property
    def values(self) -> List[int]: ...


class _ComplexReadoutValues:
    @property
    def values(self) -> List[complex]: ...


class _ReadoutValuesValues:
    @property
    def integer_values(self) -> _IntegerReadoutValues: ...
    @property
    def complex_values(self) -> _ComplexReadoutValues: ...


class _ReadoutValues:
    @property
    def values(self) -> Optional[_ReadoutValuesValues]: ...


class ReadoutMap:
    def get_readout_values(self, field: str, index: int) -> Optional[_ReadoutValues]:
        """Given a known readout field name and index, return the result's ``ReadoutValues``, if any."""
        ...
    
    def get_readout_values_for_field(self, field: str) -> Optional[List[Optional[_ReadoutValues]]]:
        """Given a known readout field name, return the result's ``ReadoutValues`` for all indices, if any."""
        ...

class QVM:
    @property
    def registers(self) -> dict: ...
    @property
    def duration(self) -> Optional[datetime.timedelta]: ...

class QPU:
    @property
    def readout_data(self) -> ReadoutMap: ...
    @property
    def duration(self) -> Optional[datetime.timedelta]: ...
