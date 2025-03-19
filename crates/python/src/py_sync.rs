/// This is a reimplementation of [`rigetti_pyo3::py_function_sync_async`] that ensures
/// Opentelemetry contexts are propagated. This could be replaced with the [`rigetti_pyo3`]
/// implementation when https://github.com/rigetti/rigetti-pyo3/issues/59 is resolved (i.e.
/// this would allow us to use [`rigetti_pyo3::py_function_sync_async`] on sync functions
/// that return a future).
macro_rules! py_function_sync_async {
     (
         $(#[$meta: meta])+
         async fn $name: ident($($(#[$arg_meta: meta])*$arg: ident : $kind: ty),* $(,)?) $(-> $ret: ty)? $body: block
     ) => {
         async fn $name($($arg: $kind,)*) $(-> $ret)? {
             $body
         }

         ::paste::paste! {
         $(#[$meta])+
         #[allow(clippy::too_many_arguments)]
         #[pyo3(name = $name "")]
         pub fn [< py_ $name >](py: ::pyo3::Python<'_> $(, $(#[$arg_meta])*$arg: $kind)*) $(-> $ret)? {
             use opentelemetry::trace::FutureExt;
             ::rigetti_pyo3::py_sync!(py, $name($($arg),*).with_current_context())
         }

         $(#[$meta])+
         #[pyo3(name = $name "_async")]
         #[allow(clippy::too_many_arguments)]
         pub fn [< py_ $name _async >](py: ::pyo3::Python<'_> $(, $(#[$arg_meta])*$arg: $kind)*) -> ::pyo3::PyResult<&::pyo3::PyAny> {
             use opentelemetry::trace::FutureExt;
             ::rigetti_pyo3::py_async!(py, $name($($arg),*).with_current_context())
         }
         }
     };
 }

pub(crate) use py_function_sync_async;
