//! Provides QVM functionality via libquil

use std::collections::HashMap;
use std::convert::TryFrom;

use crate::RegisterData;

use super::{
    http::{self, AddressRequest},
    QvmOptions,
};

/// The errors that can arise when using this client
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when calling QVM
    #[error("error when calling QVM: {0}")]
    LibquilSysQvm(#[from] libquil_sys::qvm::Error),
    /// Error when calling Quilc
    #[error("error when calling quilc: {0}")]
    LibquilSysQuilc(#[from] libquil_sys::quilc::Error),
    /// We currently only support requesting a specific set of register indices
    /// or _all_ register indices.
    #[error("can only request explicit or all indices for multishot programs")]
    UnsupportedIndicesRequestType,
    /// Error raised when trying to cast one integer type into another
    #[error("could not cast value: {0}")]
    InvalidCast(#[from] std::num::TryFromIntError),
}

impl From<Error> for super::Error {
    fn from(error: Error) -> Self {
        Self::Qvm {
            message: error.to_string(),
        }
    }
}

/// A libquil client providing QVM functionality
#[derive(Debug, Copy, Clone)]
pub struct Client;

#[async_trait::async_trait]
impl crate::qvm::Client for Client {
    async fn get_version_info(&self, _options: &QvmOptions) -> Result<String, super::Error> {
        let version = libquil_sys::qvm::get_version_info().map_err(Error::LibquilSysQvm)?;
        Ok(version.to_string())
    }

    async fn run(
        &self,
        request: &http::MultishotRequest,
        _options: &QvmOptions,
    ) -> Result<http::MultishotResponse, super::Error> {
        let program = request
            .compiled_quil
            .parse()
            .map_err(Error::LibquilSysQuilc)?;
        let addresses = request
            .addresses
            .iter()
            .map(|(address, indices)| match indices {
                AddressRequest::Indices(indices) => Ok((
                    address.clone(),
                    libquil_sys::qvm::MultishotAddressRequest::Indices(indices.clone()),
                )),
                AddressRequest::IncludeAll => Ok((
                    address.clone(),
                    libquil_sys::qvm::MultishotAddressRequest::All,
                )),
                AddressRequest::ExcludeAll => Err(Error::UnsupportedIndicesRequestType),
            })
            .collect::<Result<_, _>>()?;
        let result =
            libquil_sys::qvm::multishot(&program, addresses, i32::from(request.trials.get()))
                .map_err(Error::LibquilSysQvm)?;
        let mut registers = HashMap::new();
        for (address, values) in result {
            registers.insert(
                address,
                RegisterData::I8(
                    values
                        .iter()
                        .map(|v| v.iter().map(|i| i8::try_from(*i)).collect::<Result<_, _>>())
                        .collect::<Result<_, _>>()
                        .map_err(Error::InvalidCast)?,
                ),
            );
        }
        Ok(http::MultishotResponse { registers })
    }

    async fn run_and_measure(
        &self,
        request: &http::MultishotMeasureRequest,
        _options: &QvmOptions,
    ) -> Result<Vec<Vec<i64>>, super::Error> {
        let program = request
            .compiled_quil
            .parse()
            .map_err(Error::LibquilSysQuilc)?;
        let qubits = request.qubits.clone();
        let qubits = qubits
            .into_iter()
            .map(i32::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::InvalidCast)?;
        let result = libquil_sys::qvm::multishot_measure(
            &program,
            qubits.as_slice(),
            i32::from(request.trials.get()),
        )
        .map_err(Error::LibquilSysQvm)?;
        let result = result
            .into_iter()
            .map(|i| i.into_iter().map(i64::from).collect())
            .collect();
        Ok(result)
    }

    async fn measure_expectation(
        &self,
        request: &http::ExpectationRequest,
        _options: &QvmOptions,
    ) -> Result<Vec<f64>, super::Error> {
        let program = request
            .state_preparation
            .parse()
            .map_err(Error::LibquilSysQuilc)?;
        let operators = request
            .operators
            .iter()
            .map(|s| s.parse().map_err(Error::LibquilSysQuilc))
            .collect::<Result<Vec<_>, _>>()?;
        let operators = operators.iter().collect();
        let result =
            libquil_sys::qvm::expectation(&program, operators).map_err(Error::LibquilSysQvm)?;
        Ok(result)
    }

    async fn get_wavefunction(
        &self,
        request: &http::WavefunctionRequest,
        _options: &QvmOptions,
    ) -> Result<Vec<u8>, super::Error> {
        let program = request
            .compiled_quil
            .parse()
            .map_err(Error::LibquilSysQuilc)?;
        let amplitudes = libquil_sys::qvm::wavefunction(&program).map_err(Error::LibquilSysQvm)?;
        let amplitudes = amplitudes
            .into_iter()
            .flat_map(|c| vec![c.re, c.im])
            .flat_map(f64::to_be_bytes)
            .collect();
        Ok(amplitudes)
    }
}
