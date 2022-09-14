use std::collections::HashMap;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use edge::{convert_edges, Edge, Id};
use qcs_api::models::InstructionSetArchitecture;
use qubit::Qubit;

use crate::qpu::quilc::isa::qubit::FrbSim1q;

mod edge;
mod operator;
mod qubit;

/// Restructuring of an [`InstructionSetArchitecture`] for sending to quilc
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Compiler {
    #[serde(rename = "1Q")]
    qubits: HashMap<String, Qubit>,
    #[serde(rename = "2Q")]
    edges: HashMap<String, Edge>,
}

impl TryFrom<InstructionSetArchitecture> for Compiler {
    type Error = Error;

    fn try_from(isa: InstructionSetArchitecture) -> Result<Self, Error> {
        let architecture = isa.architecture;
        let mut qubits = Qubit::from_nodes(&architecture.nodes);

        let mut edges = convert_edges(&architecture.edges)?;

        let site_ops = isa
            .instructions
            .iter()
            .flat_map(|op| op.sites.iter().map(move |site| (op, site)));
        let frb_sim_1q = FrbSim1q::try_from(isa.benchmarks)?;

        for (op, site) in site_ops {
            match (&op.node_count, &site.node_ids.len()) {
                (Some(1), 1) => {
                    let id = &site.node_ids[0];
                    let qubit = qubits
                        .get_mut(id)
                        .ok_or_else(|| Error::QubitDoesNotExist(String::from(&op.name), *id))?;
                    qubit.add_operation(&op.name, &site.characteristics, &frb_sim_1q)?;
                }
                (Some(2), 2) => {
                    let id = Id::try_from(&site.node_ids)?;
                    let edge = edges
                        .get_mut(&id)
                        .ok_or_else(|| Error::EdgeDoesNotExist(String::from(&op.name), id))?;
                    edge.add_operation(&op.name, &site.characteristics)?;
                }
                (node_count, node_ids) => {
                    return Err(Error::IncorrectNodes(
                        (*node_count, *node_ids),
                        String::from(&op.name),
                        site.node_ids.clone(),
                    ))
                }
            }
        }

        let qubits = qubits
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        let edges = edges.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        Ok(Self { qubits, edges })
    }
}

/// All the errors that can occur from within this module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation {0} is defined for Qubit {1} but that Qubit does not exist")]
    QubitDoesNotExist(String, i32),
    #[error("Operation {0} is defined for Edge {1} but that Edge does not exist")]
    EdgeDoesNotExist(String, Id),
    #[error(
        "The number of nodes for an operation and site_operation must be (1, 1) or (2, 2). \
                        Got {0:?} while parsing operation {1} at site {2:?}"
    )]
    IncorrectNodes((Option<i32>, usize), String, Vec<i32>),
    #[error(transparent)]
    Qubit(#[from] qubit::Error),
    #[error(transparent)]
    Edge(#[from] edge::Error),
}

#[cfg(test)]
mod describe_compiler_isa {
    use std::{convert::TryFrom, fs::read_to_string};

    use float_cmp::{approx_eq, F64Margin};
    use qcs_api::models::InstructionSetArchitecture;
    use serde_json::Value;

    use super::Compiler;

    /// Compare two JSON values and make sure they are equivalent while allowing for some precision
    /// loss in numbers.
    ///
    /// Return Ok if equivalent, or tuple containing the differing elements.
    fn json_is_equivalent<'a>(
        first: &'a Value,
        second: &'a Value,
    ) -> Result<(), (&'a Value, &'a Value)> {
        let equal = match (first, second) {
            (Value::Number(first_num), Value::Number(second_num)) => {
                if !first_num.is_f64() || !second_num.is_f64() {
                    first_num == second_num
                } else {
                    let first_f64 = first_num.as_f64().unwrap();
                    let second_f64 = second_num.as_f64().unwrap();
                    approx_eq!(
                        f64,
                        first_f64,
                        second_f64,
                        F64Margin {
                            ulps: 1,
                            epsilon: 0.000_000_1
                        }
                    )
                }
            }
            (Value::Object(first_map), Value::Object(second_map)) => {
                let mut found_missing = false;
                for (key, first_value) in first_map {
                    let second_value = second_map.get(key);
                    if second_value.is_none() {
                        found_missing = true;
                        break;
                    }
                    let cmp = json_is_equivalent(first_value, second_value.unwrap());
                    cmp?;
                }
                !found_missing
            }
            (Value::Array(first_array), Value::Array(second_array))
                if first_array.len() != second_array.len() =>
            {
                false
            }
            (Value::Array(first_array), Value::Array(second_array)) => {
                let error = first_array.iter().zip(second_array).find(
                    |(first_value, second_value)| -> bool {
                        json_is_equivalent(first_value, second_value).is_err()
                    },
                );
                if let Some(values) = error {
                    return Err(values);
                }
                true
            }
            (first, second) => first == second,
        };
        if equal {
            Ok(())
        } else {
            Err((first, second))
        }
    }

    #[test]
    fn it_correctly_converts_aspen_8() {
        let input = read_to_string("tests/qcs-isa-Aspen-8.json")
            .expect("Could not read Aspen 8 input data");
        let expected_json = read_to_string("tests/compiler-isa-Aspen-8.json")
            .expect("Could not read Aspen 8 output data");
        let qcs_isa: InstructionSetArchitecture =
            serde_json::from_str(&input).expect("Could not deserialize Aspen-8 input");
        let expected: serde_json::Value =
            serde_json::from_str(&expected_json).expect("Could not deserialize Aspen-8 output");

        let compiler_isa =
            Compiler::try_from(qcs_isa).expect("Could not convert ISA to CompilerIsa");
        let serialized =
            serde_json::to_value(&compiler_isa).expect("Unable to serialize CompilerIsa");

        let result = json_is_equivalent(&serialized, &expected);
        result.expect("JSON was not equivalent");
    }
}
