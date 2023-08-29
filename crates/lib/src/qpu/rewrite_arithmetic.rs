//! This module provides functions to rewrite arithmetic in programs where
//! it is required to do so for Rigetti QPUs to understand that program.

use std::{collections::HashMap, convert::TryFrom};

use indexmap::set::IndexSet;
use num::complex::Complex64;
use quil_rs::{
    expression::{Expression, InfixExpression, InfixOperator},
    instruction::{
        AttributeValue, FrameIdentifier, Gate, Instruction, MemoryReference, ScalarType,
        SetFrequency, SetPhase, SetScale, ShiftFrequency, ShiftPhase, Vector,
    },
    program::{FrameSet, MemoryRegion},
    quil::{Quil, ToQuilError},
    Program,
};

use crate::executable::Parameters;

/// A function for converting `program` into a form that Rigetti QPUs can understand.
///
/// # Unit Conversion
///
/// QPUs expect a different unit space than Quil does. For example, gate parameters are given in
/// radians in Quil, but should be in "rotations" for QPUs where, essentially, 1 == 2π.
/// This function will convert all units to a form that QPUs can understand.
///
/// # Arithmetic Simplification
///
/// QPUs are capable of only a very limited subset of arithmetic operations. Therefore, __all__
/// arithmetic expressions will be substituted for parameters which will be precalculated
/// locally.
///
/// # Example
///
/// ```quil
/// DECLARE theta REAL
/// RZ(theta * 1.5) 0
/// ```
///
/// will be converted to something like
///
/// ```quil
/// DECLARE __SUBST REAL[1]
/// DECLARE theta REAL[1]
/// RZ(__SUBST[0]) 0
/// ```
///
/// where `__SUBST[0]` will be recalculated for each parameter set that is run and passed as a
/// distinct parameter from theta. Note that the value of `__SUBST[0]` will actually be
/// `theta * 1.5 / 2π`.
pub fn rewrite_arithmetic(program: Program) -> Result<(Program, Substitutions), Error> {
    #[cfg(feature = "tracing")]
    tracing::debug!("rewriting arithmetic");

    let mut substitutions = Substitutions::new();
    let mut new_program = program.clone_without_body_instructions();
    let instructions = program.into_body_instructions();

    let instructions = instructions
        .into_iter()
        .map(|instruction| {
            process_instruction(instruction, &mut substitutions, &new_program.frames)
        })
        .collect::<Result<Vec<Instruction>, Error>>()?;

    if !substitutions.is_empty() {
        new_program.memory_regions.insert(
            String::from(SUBSTITUTION_NAME),
            MemoryRegion {
                size: Vector {
                    data_type: ScalarType::Real,
                    length: substitutions.len() as u64,
                },
                sharing: None,
            },
        );
    }

    new_program.add_instructions(instructions);

    Ok((new_program, substitutions))
}

/// Take the user-provided map of [`Parameters`] and produce the map of substitutions which
/// should be given to QCS with the executable.
///
/// # Example
///
/// If there was a Quil program:
///
/// ```quil
/// DECLARE theta REAL
///
/// RX(theta) 0
/// RX(theta + 1) 0
/// RX(theta + 2) 0
/// ```
///
/// It would be converted  (in [`Execution::new`]) to something like:
///
/// ```quil
/// DECLARE __SUBST REAL[2]
/// DECLARE theta REAL[1]
///
/// RX(theta) 0
/// RX(__SUBST[0]) 0
/// RX(__SUBST[1]) 0
/// ```
///
/// Because QPUs do not evaluate expressions themselves. This function creates the values for
/// `__SUBST` by calculating the original expressions given the user-provided params (in this
/// case just `theta`).
pub fn get_substitutions(
    substitutions: &Substitutions,
    params: &Parameters,
) -> Result<Parameters, String> {
    // Convert into the format that quil-rs expects.
    let params: HashMap<&str, Vec<f64>> = params
        .iter()
        .map(|(key, value)| (key.as_ref(), value.clone()))
        .collect();
    let values = substitutions
        .iter()
        .map(|substitution: &Expression| {
            substitution
                .evaluate(&HashMap::new(), &params)
                .map_err(|e| {
                    format!(
                        "Could not evaluate expression {}: {e:?}",
                        substitution.to_quil_or_debug()
                    )
                })
                .and_then(|complex| {
                    if complex.im == 0.0 {
                        Ok(complex.re)
                    } else {
                        Err(String::from(
                            "Cannot substitute imaginary numbers for QPU execution",
                        ))
                    }
                })
        })
        .collect::<Result<Vec<f64>, String>>()?;
    // Convert back to the format that this library expects
    let mut patch_values: Parameters = params
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect();
    patch_values.insert(SUBSTITUTION_NAME.into(), values);
    Ok(patch_values)
}

/// All of the errors that can occur in this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A DEFRAME is missing for the given frame.
    #[error("No DEFFRAME for {0}")]
    MissingDefFrame(String),

    /// The SAMPLE-RATE value is invalid for the given frame.
    #[error("Unable to use SAMPLE-RATE {sample_rate} for frame {frame}")]
    InvalidSampleRate {
        /// The invalid SAMPLE-RATE value.
        sample_rate: String,
        /// The given frame name.
        frame: String,
    },

    /// A SAMPLE-RATE is missing for the given frame.
    #[error("SAMPLE-RATE is required for frame {0}")]
    MissingSampleRate(String),
}

pub(crate) const SUBSTITUTION_NAME: &str = "__SUBST";

fn process_instruction(
    instruction: Instruction,
    substitutions: &mut Substitutions,
    frames: &FrameSet,
) -> Result<Instruction, Error> {
    match instruction {
        Instruction::Gate(gate) => Ok(process_gate(gate, substitutions)),
        Instruction::SetScale(set_scale) => Ok(process_set_scale(set_scale, substitutions)),
        Instruction::ShiftFrequency(ShiftFrequency { frequency, frame }) => {
            process_frequency_expression(frequency, &frame, frames, substitutions).map(
                |expression| {
                    Instruction::ShiftFrequency(ShiftFrequency {
                        frame,
                        frequency: expression,
                    })
                },
            )
        }
        Instruction::SetFrequency(SetFrequency { frequency, frame }) => {
            process_frequency_expression(frequency, &frame, frames, substitutions).map(
                |expression| {
                    Instruction::SetFrequency(SetFrequency {
                        frame,
                        frequency: expression,
                    })
                },
            )
        }
        Instruction::SetPhase(SetPhase { frame, phase }) => Ok(Instruction::SetPhase(SetPhase {
            frame,
            phase: process_phase(phase, substitutions),
        })),
        Instruction::ShiftPhase(ShiftPhase { frame, phase }) => {
            Ok(Instruction::ShiftPhase(ShiftPhase {
                frame,
                phase: process_phase(phase, substitutions),
            }))
        }
        instruction => Ok(instruction),
    }
}

fn process_gate(mut gate: Gate, substitutions: &mut Substitutions) -> Instruction {
    gate.parameters = gate
        .parameters
        .into_iter()
        .map(|mut expression| {
            expression = expression.into_simplified();
            if matches!(expression, Expression::Number(_)) {
                return expression;
            }
            // exp => exp / 2π but in a way that can be simplified
            expression = divide_2_pi(expression);
            substitution(expression, substitutions)
        })
        .collect();
    Instruction::Gate(gate)
}

fn divide_2_pi(expression: Expression) -> Expression {
    Expression::Infix(InfixExpression {
        left: Box::new(expression),
        operator: InfixOperator::Slash,
        right: Box::new(Expression::Infix(InfixExpression {
            left: Box::new(Expression::Number(Complex64::from(2.0))),
            operator: InfixOperator::Star,
            right: Box::new(Expression::PiConstant),
        })),
    })
    .into_simplified()
}

fn process_set_scale(mut set_scale: SetScale, substitutions: &mut Substitutions) -> Instruction {
    set_scale.scale = set_scale.scale.into_simplified();
    if matches!(set_scale.scale, Expression::Number(_)) {
        return Instruction::SetScale(set_scale);
    }

    let SetScale { frame, scale } = set_scale;

    let expression = Expression::Infix(InfixExpression {
        left: Box::new(scale),
        operator: InfixOperator::Slash,
        right: Box::new(Expression::Number(Complex64::from(8.0))),
    })
    .into_simplified();

    Instruction::SetScale(SetScale {
        frame,
        scale: substitution(expression, substitutions),
    })
}

/// Substitute the expression (as necessary) for a SET-FREQUENCY or SHIFT-FREQUENCY instruction
fn process_frequency_expression(
    mut expression: Expression,
    frame: &FrameIdentifier,
    frames: &FrameSet,
    substitutions: &mut Substitutions,
) -> Result<Expression, Error> {
    expression = expression.into_simplified();
    if matches!(expression, Expression::Number(_)) {
        return Ok(expression);
    }
    let attributes = frames
        .get(frame)
        .ok_or_else(|| Error::MissingDefFrame(frame.name.clone()))?;
    let sample_rate = match attributes.get("SAMPLE-RATE") {
        Some(AttributeValue::Expression(expression)) => expression,
        Some(AttributeValue::String(sample_rate)) => {
            return Err(Error::InvalidSampleRate {
                sample_rate: sample_rate.clone(),
                frame: frame.name.clone(),
            });
        }
        None => {
            return Err(Error::MissingSampleRate(frame.name.clone()));
        }
    };
    if let Some(AttributeValue::Expression(center_frequency)) = attributes.get("CENTER-FREQUENCY") {
        expression = Expression::Infix(InfixExpression {
            left: Box::new(expression),
            operator: InfixOperator::Minus,
            right: Box::new(center_frequency.clone()),
        });
    }
    expression = Expression::Infix(InfixExpression {
        left: Box::new(expression),
        operator: InfixOperator::Slash,
        right: Box::new(sample_rate.clone()),
    });
    Ok(substitution(expression, substitutions))
}

fn process_phase(phase: Expression, substitutions: &mut Substitutions) -> Expression {
    if matches!(phase, Expression::Number(_)) {
        return phase;
    }

    let expression = divide_2_pi(phase);

    substitution(expression, substitutions)
}

/// Take an expression and produce or return the existing substitution for it, recording that
/// substitution in `substitutions`.
fn substitution(expression: Expression, substitutions: &mut Substitutions) -> Expression {
    let index = substitutions.get_index_of(&expression).unwrap_or_else(|| {
        substitutions.insert(expression);
        substitutions.len() - 1
    });
    let reference = MemoryReference {
        name: String::from(SUBSTITUTION_NAME),
        index: index as u64,
    };

    Expression::Address(reference)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RewrittenProgram {
    inner: Program,
    pub(crate) substitutions: Substitutions,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct RewrittenQuil(pub(crate) String);

impl From<RewrittenQuil> for String {
    fn from(quil: RewrittenQuil) -> String {
        quil.0
    }
}

impl TryFrom<Program> for RewrittenProgram {
    type Error = Error;

    fn try_from(program: Program) -> Result<Self, Self::Error> {
        let (inner, substitutions) = rewrite_arithmetic(program)?;
        Ok(Self {
            inner,
            substitutions,
        })
    }
}

impl RewrittenProgram {
    pub(crate) fn to_string(&self) -> Result<RewrittenQuil, ToQuilError> {
        Ok(RewrittenQuil(self.inner.to_quil()?))
    }
}

#[cfg(test)]
mod describe_rewrite_arithmetic {
    use std::str::FromStr;

    use quil_rs::{quil::Quil, Program};

    use crate::qpu::rewrite_arithmetic::rewrite_arithmetic;

    #[test]
    fn it_substitutes_gate_parameters() {
        let program = Program::from_str("DECLARE theta REAL; RZ(theta) 0").unwrap();
        let expected =
            Program::from_str("DECLARE __SUBST REAL[1]; DECLARE theta REAL[1]; RZ(__SUBST[0]) 0")
                .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 1);
        insta::assert_snapshot!(substitutions[0].to_quil_or_debug());
    }

    #[test]
    fn it_leaves_literal_gates_alone() {
        let program = Program::from_str("RZ(1.0) 0").unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program.clone()).unwrap();
        assert_eq!(actual, program);
        assert_eq!(substitutions.len(), 0);
    }

    #[test]
    fn it_substitutes_and_reuses_gate_expressions() {
        let program =
            Program::from_str("DECLARE theta REAL; RZ(theta*1.5) 0; RX(theta*1.5) 0").unwrap();
        let expected = Program::from_str(
            "DECLARE __SUBST REAL[1]; DECLARE theta REAL[1]; RZ(__SUBST[0]) 0; RX(__SUBST[0]) 0",
        )
        .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 1);
        insta::assert_snapshot!(substitutions[0].to_quil_or_debug());
    }

    #[test]
    fn it_allocates_for_multiple_expressions() {
        let program = Program::from_str(
            r#"
DECLARE theta REAL
DECLARE beta REAL
RZ(3 * theta) 0
RZ(beta+theta) 0
"#,
        )
        .unwrap();
        let expected = Program::from_str(
            r#"
DECLARE __SUBST REAL[2]
DECLARE theta REAL[1]
DECLARE beta REAL[1]
RZ(__SUBST[0]) 0
RZ(__SUBST[1]) 0
"#,
        )
        .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 2);
        insta::assert_snapshot!(substitutions[0].to_quil_or_debug());
        insta::assert_snapshot!(substitutions[1].to_quil_or_debug());
    }

    #[test]
    fn it_converts_set_scale_units() {
        let program = Program::from_str(
            r#"
DECLARE theta REAL
SET-SCALE 0 "rf" 1.0
SET-SCALE 0 "rf" theta
"#,
        )
        .unwrap();
        let expected = Program::from_str(
            r#"
DECLARE __SUBST REAL[1]
DECLARE theta REAL[1]
SET-SCALE 0 "rf" 1.0
SET-SCALE 0 "rf" __SUBST[0]
"#,
        )
        .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 1);
        insta::assert_snapshot!(substitutions[0].to_quil_or_debug());
    }

    #[test]
    fn it_converts_frequency_expressions() {
        let program = Program::from_str(
            r#"
DEFFRAME 0 "rf":
    CENTER-FREQUENCY: 10.0
    SAMPLE-RATE: 20.0
DEFFRAME 1 "rf":
    SAMPLE-RATE: 20.0
DECLARE theta REAL
SET-FREQUENCY 0 "rf" theta
SHIFT-FREQUENCY 0 "rf" theta
SET-FREQUENCY 1 "rf" theta
"#,
        )
        .unwrap();
        let expected = Program::from_str(
            r#"
DEFFRAME 0 "rf":
    CENTER-FREQUENCY: 10.0
    SAMPLE-RATE: 20.0
DEFFRAME 1 "rf":
    SAMPLE-RATE: 20.0
DECLARE __SUBST REAL[2]
DECLARE theta REAL
SET-FREQUENCY 0 "rf" __SUBST[0]
SHIFT-FREQUENCY 0 "rf" __SUBST[0]
SET-FREQUENCY 1 "rf" __SUBST[1]
"#,
        )
        .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 2);
        insta::assert_snapshot!(substitutions[0].to_quil_or_debug());
        insta::assert_snapshot!(substitutions[1].to_quil_or_debug());
    }

    #[test]
    fn it_errs_when_converting_frequency_without_frame() {
        let program = Program::from_str(
            r#"
DEFFRAME 0 "rf":
    CENTER-FREQUENCY: 10.0
    SAMPLE-RATE: 20.0
DECLARE theta REAL
SET-FREQUENCY 0 "rf" theta
SHIFT-FREQUENCY 0 "rf" theta
SET-FREQUENCY 1 "rf" theta
"#,
        )
        .unwrap();
        let result = rewrite_arithmetic(program);
        assert!(result.is_err());
    }

    #[test]
    fn it_errs_when_converting_frequency_without_sample_rate() {
        let program = Program::from_str(
            r#"
DEFFRAME 0 "rf":
    CENTER-FREQUENCY: 10.0
DECLARE theta REAL
SET-FREQUENCY 0 "rf" theta
SHIFT-FREQUENCY 0 "rf" theta
"#,
        )
        .unwrap();
        let result = rewrite_arithmetic(program);
        assert!(result.is_err());
    }

    #[test]
    fn it_converts_phases() {
        let program = Program::from_str(
            r#"
DECLARE theta REAL
SET-PHASE 0 "rf" theta
SHIFT-PHASE 0 "rf" theta
"#,
        )
        .unwrap();
        let expected = Program::from_str(
            r#"
DECLARE __SUBST REAL[1]
DECLARE theta REAL[1]
SET-PHASE 0 "rf" __SUBST[0]
SHIFT-PHASE 0 "rf" __SUBST[0]
"#,
        )
        .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 1);
        insta::assert_snapshot!(substitutions[0].to_quil_or_debug());
    }
}

pub(crate) type Substitutions = IndexSet<Expression>;
