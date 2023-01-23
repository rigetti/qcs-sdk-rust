"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""

import datetime
from typing import List, Optional

class IntegerReadoutValues:
    values: List[int]


class ComplexReadoutValues:
    values: List[complex]


class ReadoutValuesValues:
    integer_values: IntegerReadoutValues
    complex_values: ComplexReadoutValues


class ReadoutValues:
    values: Optional[ReadoutValuesValues]


class ReadoutMap:
    def get_readout_values(self, field: str, index: int) -> Optional[ReadoutValues]:
        ...
    
    def get_readout_values_for_field(self, field: str) -> Optional[List[Optional[ReadoutValues]]]:
        ...

class QVM:
    registers: dict
    duration: Optional[datetime.timedelta]

class QPU:
    readout_data: ReadoutMap
    duration: Optional[datetime.timedelta]
