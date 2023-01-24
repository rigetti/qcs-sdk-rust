use enum_as_inner::EnumAsInner;
use num::complex::Complex32;
use serde::{Deserialize, Serialize};

/// Data resulting from [`Executable::execute_on_qvm`](`crate::Executable::execute_on_qvm`)
///
/// This represents a "register" of typed memory across some number of shots.
/// The register corresponds to the usage of a `DECLARE` instruction in Quil, and the name of that
/// register should be provided with [`Executable::read_from`](`crate::Executable::read_from`).
///
/// There is a variant of this enum for each type of data that a register could hold. The register
/// is represented as a 2-dimensional, array `M` where the value `M[shot_number][memory_index]`
/// represents the value at `memory_index` for `shot_number`.
///
/// # Usage
///
/// Typically you will already know what type of data the `ExecutionResult` _should_ have, so you can
/// use the [`mod@enum_as_inner`] methods (e.g. [`RegisterData::into_i8`]) in order to
/// convert any variant type to its inner data.
#[derive(Clone, Debug, Deserialize, EnumAsInner, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RegisterData {
    /// Corresponds to the Quil `BIT` or `OCTET` types.
    I8(Vec<Vec<i8>>),
    /// Corresponds to the Quil `REAL` type.
    F64(Vec<Vec<f64>>),
    /// Corresponds to the Quil `INTEGER` type.
    I16(Vec<Vec<i16>>),
    /// Results containing complex numbers.
    #[serde(skip)]
    Complex32(Vec<Vec<Complex32>>),
}
