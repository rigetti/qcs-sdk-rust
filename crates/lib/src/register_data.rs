use enum_as_inner::EnumAsInner;
use num::complex::Complex32;
use serde::{Deserialize, Serialize};

/// Data resulting from [`Executable::execute_on_qvm`](`crate::Executable::execute_on_qvm`)
///
/// This represents a single vector (or "register") of typed memory across some number of shots.
/// The register corresponds to the usage of a `DECLARE` instruction in Quil, and the name of that
/// register should be provided with [`Executable::read_from`](`crate::Executable::read_from`).
///
/// There is a variant of this enum for each type of data that a register could hold. The register
/// is represented as a 2-dimensional array `M` where the value `M[shot_number][memory_index]`
/// represents the value at `memory_index` for `shot_number`.
///
/// # Usage
///
/// Typically, you will be interacting with this data through the [`crate::ResultData`] of an
/// [`crate::ExecutionData`] returned after running a program. In those cases, you'll probably
/// want to convert it to a readout map using [`crate::ResultData.to_register_map()`]. This
/// will give you each register in the form of a [`crate::RegisterMatrix`] which is similar
/// but backed by an [`ndarray::Array2`] and more convenient for working with matrices.
///
/// If you are interacting with [`RegisterData`] directly, then you should already know what type of data it _should_
/// have, so you can  use the [`mod@enum_as_inner`] methods (e.g. [`RegisterData::into_i8`]) in order to
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
