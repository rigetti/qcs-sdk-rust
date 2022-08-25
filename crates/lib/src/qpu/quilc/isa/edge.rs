use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

use serde::{Serialize, Serializer};

use qcs_api::models;
use qcs_api::models::Characteristic;

use super::operator::{
    Arguments, Operator, OperatorMap, Parameters, PERFECT_DURATION, PERFECT_FIDELITY,
};

/// Represents a connection between two qubits.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct Edge {
    #[serde(rename = "ids")]
    id: EdgeId,
    #[serde(skip_serializing_if = "is_false")]
    dead: bool,
    gates: OperatorMap,
}

// Gross hack to not include `dead` if unneeded, to follow pyQuil's implementation
#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(val: &bool) -> bool {
    !*val
}

impl Edge {
    /// Construct an edge with no gates
    fn empty(id: EdgeId) -> Self {
        Self {
            id,
            dead: true,
            gates: OperatorMap::new(),
        }
    }

    pub(crate) fn add_operation<'op_name>(
        &mut self,
        op_name: &'op_name str,
        characteristics: &[Characteristic],
    ) -> Result<(), Error> {
        let operator = match GATE_PARAMS.get_key_value(op_name) {
            Some((key, params)) => basic_gates(key, params, characteristics),
            _ => {
                if op_name == "WILDCARD" {
                    WILDCARD
                } else {
                    return Err(Error::UnknownOperator(String::from(op_name)));
                }
            }
        };

        if self.gates.add_one(operator) {
            self.dead = false;
        }
        Ok(())
    }
}

/// All the error which can occur from within this module.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Unknown operator: {0}")]
    UnknownOperator(String),
    #[error("Edges should have exactly 2 nodes, got {0}")]
    EdgeSize(usize),
}

#[cfg(test)]
mod describe_edge {
    use super::*;

    #[test]
    fn it_skips_serializing_dead_if_false() {
        let undead_qubit = Edge {
            id: EdgeId::new([1, 2]),
            dead: false,
            gates: OperatorMap::new(),
        };
        let dead_qubit = Edge {
            id: EdgeId::new([1, 2]),
            dead: true,
            gates: OperatorMap::new(),
        };

        let expected_dead = serde_json::json!({
            "ids": [1, 2],
            "dead": true,
            "gates": []
        });
        let expected_undead = serde_json::json!({
            "ids": [1, 2],
            "gates": []
        });

        let undead = serde_json::to_value(&undead_qubit).unwrap();
        let dead = serde_json::to_value(&dead_qubit).unwrap();

        assert_eq!(undead, expected_undead);
        assert_eq!(dead, expected_dead);
    }
}

/// The unique identifier of an [`Edge`] is defined by the sorted combination
/// of the nodes that make it up. When used as a key in the map that will be sent to `quilc`, the
/// key should look like `"{node_id_1}-{node_id_2}"`
///
/// This struct enforces those things to make looking up of Edges easier when converting ISAs.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct EdgeId([i32; 2]);

impl EdgeId {
    pub(crate) fn new(mut node_ids: [i32; 2]) -> Self {
        node_ids.sort_unstable();
        Self(node_ids)
    }
}

impl TryFrom<&Vec<i32>> for EdgeId {
    type Error = Error;

    fn try_from(node_ids: &Vec<i32>) -> Result<Self, Error> {
        let node_ids: [i32; 2] = node_ids
            .as_slice()
            .try_into()
            .map_err(|_| Error::EdgeSize(node_ids.len()))?;
        Ok(Self::new(node_ids))
    }
}

impl Serialize for EdgeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0)
    }
}

impl Display for EdgeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0[0], self.0[1])
    }
}

#[cfg(test)]
mod describe_edge_id {
    use super::*;

    #[test]
    fn it_serializes_as_an_int_list() {
        let edge_id = EdgeId::new([1, 2]);
        let serialized = serde_json::to_value(&edge_id).expect("Could not serialize EdgeId");
        assert_eq!(serialized, serde_json::json!([1, 2]));
    }

    #[test]
    fn it_displays_as_quilc_style_string() {
        let edge_id = EdgeId::new([1, 2]);
        assert_eq!(edge_id.to_string(), "1-2".to_string());
    }

    #[test]
    fn it_sorts_ids_when_constructed() {
        let edge_id = EdgeId::new([2, 1]);
        assert_eq!(edge_id.to_string(), "1-2".to_string());
    }

    #[test]
    fn it_fails_to_convert_from_vec_when_wrong_size() {
        let inputs = vec![
            // No nodes
            vec![],
            // Only 1 node
            vec![1],
            // Too many nodes
            vec![1, 2, 3],
        ];
        for input in inputs {
            let result = EdgeId::try_from(&input);
            assert!(result.is_err());
        }
    }

    #[test]
    fn it_successfully_converts_from_correct_vec() {
        let input = vec![2, 1];
        let result = EdgeId::try_from(&input).expect("Failed to convert valid Vec to EdgeId");
        assert_eq!(result.to_string(), "1-2".to_string());
    }
}

/// Convert the QCS ISA representation of edges to the quilc form [`Edge`]
pub(crate) fn convert_edges(edges: &[models::Edge]) -> Result<HashMap<EdgeId, Edge>, Error> {
    edges
        .iter()
        .map(|edge| {
            let id = EdgeId::try_from(&edge.node_ids)?;
            let edge = Edge::empty(id);
            Ok((id, edge))
        })
        .collect()
}

#[cfg(test)]
mod describe_convert_edges {
    use super::*;

    #[test]
    fn it_converts_valid_edges() {
        let input = vec![
            models::Edge {
                node_ids: vec![1, 2],
            },
            models::Edge {
                node_ids: vec![2, 3],
            },
            models::Edge {
                node_ids: vec![3, 1],
            },
        ];

        let result = convert_edges(&input).expect("Could not convert valid inputs");

        assert_eq!(result.len(), 3);
        let expected_ids = [
            EdgeId::new([1, 2]),
            EdgeId::new([2, 3]),
            EdgeId::new([1, 3]),
        ];
        for expected in expected_ids {
            assert_eq!(result[&expected].id, expected)
        }
    }
}

lazy_static::lazy_static! {
    static ref GATE_PARAMS: HashMap<&'static str, GateParams> = {
        let mut m = HashMap::new();
        m.insert("CZ", GateParams{
            default_fidelity: 0.89,
            duration: 200.0,
            characteristic_name: "fCZ",
            parameters: Parameters::Empty,
        });
        m.insert("ISWAP", GateParams{
            default_fidelity: 0.90,
            duration: 200.0,
            characteristic_name: "fISWAP",
            parameters: Parameters::Empty,
        });
        m.insert("CPHASE", GateParams{
            default_fidelity: 0.85,
            duration: 200.0,
            characteristic_name: "fCPHASE",
            parameters: Parameters::Theta,
        });
        m.insert("XY", GateParams{
            default_fidelity: 0.86,
            duration: 200.0,
            characteristic_name: "fXY",
            parameters: Parameters::Theta,
        });
        m
    };
}

/// Contains everything you need to know to parse a gate on an edge for a particular op
struct GateParams {
    default_fidelity: f64,
    duration: f64,
    characteristic_name: &'static str,
    parameters: Parameters,
}

fn basic_gates(
    op_name: &'static str,
    params: &GateParams,
    characteristics: &[Characteristic],
) -> Operator {
    let GateParams {
        default_fidelity,
        duration,
        characteristic_name,
        parameters,
    } = params;
    let fidelity = characteristics
        .iter()
        .find(|characteristic| &characteristic.name == characteristic_name)
        .map_or(*default_fidelity, |characteristic| {
            characteristic.value.into()
        });

    Operator::Gate {
        operator: op_name,
        parameters: *parameters,
        arguments: Arguments::Underscores,
        fidelity,
        duration: *duration,
    }
}

const WILDCARD: Operator = Operator::Gate {
    operator: "_",
    duration: PERFECT_DURATION,
    fidelity: PERFECT_FIDELITY,
    parameters: Parameters::Underscore,
    arguments: Arguments::Underscores,
};
