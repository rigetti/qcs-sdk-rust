"""
Do not import this file, it has no exports.
It is only here to represent the structure of the rust source code 1:1
"""

from typing import List, Optional


class RegisterData:
    """
    Values present in a register that are one of a set of variants.

    Variants:
        - ``i8``: Corresponds to the Quil `BIT` or `OCTET` types.
        - ``i16``: Corresponds to the Quil `INTEGER` type.
        - ``f64``: Corresponds to the Quil `REAL` type.
        - ``complex32``: Results containing complex numbers.

    Methods (each per variant):
        - ``is_*``: if the underlying values are that type.
        - ``as_*``: if the underlying values are that type, then those values, otherwise ``None``.
        - ``to_*``: the underlying values as that type, raises ``ValueError`` if they are not.
        - ``from_*``: wrap underlying values as this enum type.

    """
    
    def is_i8() -> bool: ...
    def is_i16() -> bool: ...
    def is_f64() -> bool: ...
    def is_complex32() -> bool: ...

    def as_i8() -> Optional[List[List[int]]]: ...
    def as_i16() -> Optional[List[List[int]]]: ...
    def as_f64() -> Optional[List[List[float]]]: ...
    def as_complex32() -> Optional[List[List[complex]]]: ...

    def to_i8() -> List[List[int]]: ...
    def to_i16() -> List[List[int]]: ...
    def to_f64() -> List[List[float]]: ...
    def to_complex32() -> List[List[complex]]: ...

    def from_i8(inner: List[List[int]]) -> "RegisterData": ...
    def from_i16(inner: List[List[int]]) -> "RegisterData": ...
    def from_f64(inner: List[List[float]]) -> "RegisterData": ...
    def from_complex32(inner: List[List[complex]]) -> "RegisterData": ...
