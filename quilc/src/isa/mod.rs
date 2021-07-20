use std::collections::HashMap;
use std::convert::TryFrom;

use serde::Serialize;

use edge::Edge;
use eyre::{eyre, Report, Result};
use qcs_api::models::InstructionSetArchitecture;

use crate::isa::edge::{convert_edges, EdgeId};
use crate::isa::operator::Operator;
use crate::isa::qubit::Qubit;

mod edge;
mod operator;
mod qubit;

/// Restructuring of a [`models::InstructionSetArchitecture`] for sending to quilc
#[derive(Serialize, Debug)]
pub(crate) struct CompilerIsa {
    #[serde(rename = "1Q")]
    qubits: HashMap<String, Qubit>,
    #[serde(rename = "2Q")]
    edges: HashMap<String, Edge>,
}

impl TryFrom<&InstructionSetArchitecture> for CompilerIsa {
    type Error = Report;

    fn try_from(isa: &InstructionSetArchitecture) -> Result<Self> {
        let mut qubits = Qubit::from_nodes(&isa.architecture.nodes);

        let mut edges = convert_edges(&isa.architecture.edges)?;

        let site_ops = isa
            .instructions
            .iter()
            .flat_map(|op| op.sites.iter().map(move |site| (op, site)));

        for (op, site) in site_ops {
            match (&op.node_count, &site.node_ids.len()) {
                (Some(1), 1) => {
                    let id = &site.node_ids[0];
                    let qubit = qubits.get_mut(id).ok_or_else(
                        || eyre!("Operation {} is defined for Qubit {} but that Qubit does not exist", op.name, id)
                    )?;
                    qubit.add_operation(&op.name, &site.characteristics, &isa.benchmarks)?;
                }
                (Some(2), 2) => {
                    let id = EdgeId::try_from(&site.node_ids)?;
                    let edge = edges.get_mut(&id).ok_or_else(
                        || eyre!("Operation {} is defined for Edge {} but that Edge does not exist", op.name, id)
                    )?;
                    edge.add_operation(&op.name, &site.characteristics)?;
                }
                item => {
                    return Err(eyre!(
                        "The number of nodes for an operation and site_operation must be (1, 1) or (2, 2). \
                        Got {:?} while parsing operation {} at site {:?}", item, op.name, site.node_ids
                    ))
                },
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

#[cfg(test)]
mod describe_compiler_isa {
    use super::*;
    use float_cmp::{approx_eq, F64Margin};
    use serde_json::Value;
    use std::fs::read_to_string;

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
                            epsilon: 0.0000001
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
                    if !cmp.is_ok() {
                        return cmp;
                    }
                }
                !found_missing
            }
            (Value::Array(first_array), Value::Array(second_array)) => {
                if first_array.len() != second_array.len() {
                    false
                } else {
                    let error = first_array.into_iter().zip(second_array).find(
                        |(first_value, second_value)| {
                            json_is_equivalent(first_value, second_value).is_err()
                        },
                    );
                    if let Some(values) = error {
                        return Err(values);
                    }
                    true
                }
            }
            (first, second) => first == second,
        };
        if !equal {
            Err((first, second))
        } else {
            Ok(())
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
            CompilerIsa::try_from(&qcs_isa).expect("Could not convert ISA to CompilerIsa");
        let serialized =
            serde_json::to_value(&compiler_isa).expect("Unable to serialize CompilerIsa");

        let result = json_is_equivalent(&serialized, &expected);
        result.expect("JSON was not equivalent");
    }
}
