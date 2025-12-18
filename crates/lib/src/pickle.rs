// Copyright 2025 Rigetti Computing
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Implement `__new__` and `__getnewargs__` for the common cases.
///
/// # Purpose
///
/// The `__getnewargs__` method is used by Python's `copy` and `pickle` modules,
/// hence why the name: this enables a type to be (un)pickled and (deep)copied.
///
/// # Usage
///
/// The implementation generated simply `clone`s all the fields names given to the macro,
/// though you can customize the body of the `__new__` implementation, if desired.
/// You can choose the name of the `__new__` method and its visibility,
/// so the easiest way to use this macro with a `#[pyclass]` that you also use in Rust is like:
///
/// ```ignore
/// pub struct Foo {
///     bar: i32,
///     baz: String,
/// }
///
/// pickleable_new! {
///     impl Foo {
///         pub fn new(bar: i32, baz: String);
///     }
/// }
/// ```
///
/// That will expand to the default body, namely:
///
/// ```ignore
/// #[pymethods]
/// impl Foo {
///     pub fn new(bar: i32, baz: String) -> Self {
///         Self {
///             bar,
///             baz,
///         }
///     }
///
///     fn __getnewargs__(&self) -> (i32, String) {
///         (
///             self.bar.clone(),
///             self.baz.clone(),
///         )
///     }
/// }
/// ```
///
/// You can supply a body for the constructor if you need other logic, such as validation;
/// however, if you want to accept different parameter names or type than the struct's fields,
/// you'll have to tell the macro the struct's field names and return types.
///
/// ```ignore
/// pickleable_new! {
///     impl Foo {
///         pub fn new(bar: i32, baz: &str) -> Self {
///             Self {
///                 bar,
///                 baz: baz.to_string(),
///             }
///         }
///     }
/// }
/// ```
///
/// That will generate the same `__getnewargs__` implementation, but use your given `new` body.
///
/// # Note
///
/// This macro expands to two `impl` blocks: one for if the `python` feature is enabled,
/// which includes the `#[pymethods]` block with its `#[new]` and `__getnewargs__` methods,
/// and a second that for when the `python` feature is not enabled,
/// which simply implements the constructor.
macro_rules! pickleable_new {
    // Default implementation: just list the fields and types, and this will do the rest.
    (
        $(#[$impl_meta:meta])*
        impl $name:ident {
            $(#[$meta:meta])*
            $pub:vis fn $new:ident( $($field:ident: $field_type:ty$(,)?)*);
        }
    ) => {
        pickleable_new! {
            $(#[$impl_meta])*
            impl $name {
                $(#[$meta])*
                $pub fn $new($($field: $field_type,)*) -> $name {
                    Self {
                        $($field,)*
                    }
                }
            }
        }
    };

    // If __new__ needs actual logic, you can supply a body.
    (
        $(#[$impl_meta:meta])*
        impl $name:ident {
            $(#[$meta:meta])*
            $pub:vis fn $new:ident( $($field:ident: $field_type:ty$(,)?)*) -> $ret:ty {
                $($body:tt)+
            }
        }
    ) => {
        $(#[$impl_meta])*
        #[cfg(feature = "python")]
        #[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[pyo3::pymethods]
        impl $name {
            $(#[$meta])*
            #[new]
            $pub fn $new($($field: $field_type,)*) -> $ret {
                $($body)+
            }

            fn __getnewargs__(&self) -> ($($field_type,)*) {
                (
                    $((self.$field).clone(),)*
                )
            }
        }

        $(#[$impl_meta])*
        #[cfg(not(feature = "python"))]
        impl $name {
            $(#[$meta])*
            $pub fn $new($($field: $field_type,)*) -> $ret {
                $($body)+
            }
        }
    };
}

pub(crate) use pickleable_new;
