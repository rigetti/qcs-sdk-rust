use std::convert::TryFrom;

use eyre::{eyre, Report, Result};
use indexmap::set::IndexSet;
use num::complex::Complex64;
use quil::expression::{Expression, InfixOperator};
use quil::instruction::{
    AttributeValue, FrameIdentifier, Gate, Instruction, MemoryReference, ScalarType, SetFrequency,
    SetPhase, SetScale, ShiftFrequency, ShiftPhase, Vector,
};
use quil::program::{FrameSet, MemoryRegion};
use quil::Program;

use crate::qpu::quilc::NativeQuilProgram;

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
pub(crate) fn rewrite_arithmetic(program: Program) -> Result<(Program, Substitutions)> {
    let mut substitutions = Substitutions::new();
    let Program {
        calibrations,
        frames,
        mut memory_regions,
        waveforms,
        instructions,
    } = program;

    let instructions = instructions
        .into_iter()
        .map(|instruction| process_instruction(instruction, &mut substitutions, &frames))
        .collect::<Result<Vec<Instruction>>>()?;

    if !substitutions.is_empty() {
        memory_regions.insert(
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

    Ok((
        Program {
            calibrations,
            frames,
            memory_regions,
            waveforms,
            instructions,
        },
        substitutions,
    ))
}

pub(crate) const SUBSTITUTION_NAME: &str = "__SUBST";

fn process_instruction(
    instruction: Instruction,
    substitutions: &mut Substitutions,
    frames: &FrameSet,
) -> Result<Instruction> {
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
            expression = expression.simplify();
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
    Expression::Infix {
        left: Box::new(expression),
        operator: InfixOperator::Slash,
        right: Box::new(Expression::Infix {
            left: Box::new(Expression::Number(Complex64::from(2.0))),
            operator: InfixOperator::Star,
            right: Box::new(Expression::PiConstant),
        }),
    }
    .simplify()
}

fn process_set_scale(mut set_scale: SetScale, substitutions: &mut Substitutions) -> Instruction {
    set_scale.scale = set_scale.scale.simplify();
    if matches!(set_scale.scale, Expression::Number(_)) {
        return Instruction::SetScale(set_scale);
    }

    let SetScale { frame, scale } = set_scale;

    let expression = Expression::Infix {
        left: Box::new(scale),
        operator: InfixOperator::Slash,
        right: Box::new(Expression::Number(Complex64::from(8.0))),
    }
    .simplify();

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
) -> Result<Expression> {
    expression = expression.simplify();
    if matches!(expression, Expression::Number(_)) {
        return Ok(expression);
    }
    let attributes = frames
        .get(frame)
        .ok_or_else(|| eyre!("No DEFFRAME for {}", frame))?;
    let sample_rate = match attributes.get("SAMPLE-RATE") {
        Some(AttributeValue::Expression(expression)) => expression,
        Some(other) => {
            return Err(eyre!(
                "Unable to use SAMPLE-RATE {} for frame {}",
                other,
                frame
            ));
        }
        None => {
            return Err(eyre!("SAMPLE-RATE is required for frame {}", frame));
        }
    };
    if let Some(AttributeValue::Expression(center_frequency)) = attributes.get("CENTER-FREQUENCY") {
        expression = Expression::Infix {
            left: Box::new(expression),
            operator: InfixOperator::Minus,
            right: Box::new(center_frequency.clone()),
        }
    }
    expression = Expression::Infix {
        left: Box::new(expression),
        operator: InfixOperator::Slash,
        right: Box::new(sample_rate.clone()),
    };
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

pub(crate) struct RewrittenProgram {
    inner: Program,
    pub(crate) substitutions: Substitutions,
}

pub(crate) struct RewrittenQuil(String);

impl From<RewrittenQuil> for String {
    fn from(quil: RewrittenQuil) -> String {
        quil.0
    }
}

impl TryFrom<NativeQuilProgram> for RewrittenProgram {
    type Error = Report;

    fn try_from(program: NativeQuilProgram) -> Result<Self> {
        let (inner, substitutions) = rewrite_arithmetic(program.into())?;
        Ok(Self {
            inner,
            substitutions,
        })
    }
}

impl RewrittenProgram {
    pub(crate) fn to_string(&self) -> RewrittenQuil {
        RewrittenQuil(self.inner.to_string(true))
    }
}

#[cfg(test)]
mod describe_rewrite_arithmetic {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn it_substitutes_gate_parameters() {
        let program = Program::from_str("DECLARE theta REAL; RZ(theta) 0").unwrap();
        let expected =
            Program::from_str("DECLARE __SUBST REAL[1]; DECLARE theta REAL[1]; RZ(__SUBST[0]) 0")
                .unwrap();
        let (actual, substitutions) = rewrite_arithmetic(program).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(substitutions.len(), 1);
        assert_eq!(substitutions[0].to_string(), "(theta[0]/6.283185307179586)");
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
        assert_eq!(
            substitutions[0].to_string(),
            "((theta[0]*1.5)/6.283185307179586)"
        );
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
        assert_eq!(
            substitutions[0].to_string(),
            "((3*theta[0])/6.283185307179586)"
        );
        assert_eq!(
            substitutions[1].to_string(),
            "((beta[0]+theta[0])/6.283185307179586)"
        );
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
        assert_eq!(substitutions[0].to_string(), "(theta[0]/8)");
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
        assert_eq!(substitutions[0].to_string(), "((theta[0]-10)/20)");
        assert_eq!(substitutions[1].to_string(), "(theta[0]/20)");
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
        assert_eq!(substitutions[0].to_string(), "(theta[0]/6.283185307179586)");
    }
}

pub(crate) type Substitutions = IndexSet<Expression>;
