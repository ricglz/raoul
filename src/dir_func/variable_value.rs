use std::fmt;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Not, Sub};

use crate::{ast::ast_kind::AstNodeKind, enums::Types};

#[derive(Clone, PartialEq)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl VariableValue {
    pub fn is_number(&self) -> bool {
        match self {
            Self::Integer(_) | Self::Float(_) | Self::String(_) => true,
            _ => false,
        }
    }

    #[inline]
    fn cast_to_bool(&self) -> VariableValue {
        Self::Bool(bool::from(self))
    }

    #[inline]
    fn cast_to_float(&self) -> VariableValue {
        Self::Float(f64::from(self))
    }

    pub fn cast_to(&self, to: Types) -> VariableValue {
        match to {
            Types::BOOL => self.cast_to_bool(),
            Types::FLOAT => self.cast_to_float(),
            _ => self.clone(),
        }
    }
}

impl From<&VariableValue> for Types {
    fn from(v: &VariableValue) -> Self {
        match v {
            VariableValue::Integer(_) => Types::INT,
            VariableValue::Float(_) => Types::FLOAT,
            VariableValue::String(_) => Types::STRING,
            VariableValue::Bool(_) => Types::BOOL,
        }
    }
}

impl From<AstNodeKind<'_>> for VariableValue {
    fn from(v: AstNodeKind) -> Self {
        match v {
            AstNodeKind::Integer(value) => VariableValue::Integer(value),
            AstNodeKind::Float(value) => VariableValue::Float(value),
            AstNodeKind::String(value) => VariableValue::String(value.clone()),
            AstNodeKind::Bool(value) => VariableValue::Bool(value),
            _ => unreachable!(),
        }
    }
}

impl From<VariableValue> for f64 {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::Integer(a) => a.to_string().parse().unwrap(),
            VariableValue::Float(a) => a,
            VariableValue::String(a) => a.parse().unwrap(),
            _ => unreachable!(),
        }
    }
}

impl From<&VariableValue> for f64 {
    fn from(v: &VariableValue) -> Self {
        Self::from(v.to_owned())
    }
}

impl From<VariableValue> for bool {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::Integer(a) => a != 0,
            VariableValue::Bool(a) => a,
            _ => unreachable!(),
        }
    }
}

impl From<&VariableValue> for bool {
    fn from(v: &VariableValue) -> Self {
        Self::from(v.to_owned())
    }
}

impl fmt::Debug for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            VariableValue::Bool(value) => value.to_string(),
            VariableValue::Integer(value) => value.to_string(),
            VariableValue::Float(value) => value.to_string(),
            VariableValue::String(value) => value.to_owned(),
        };
        write!(f, "{}", value)
    }
}

impl Add for VariableValue {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Self::Integer(a + b)
        } else {
            Self::Float(f64::from(self) + f64::from(other))
        }
    }
}

impl Sub for VariableValue {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Self::Integer(a - b)
        } else {
            Self::Float(f64::from(self) - f64::from(other))
        }
    }
}

impl Mul for VariableValue {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Self::Integer(a * b)
        } else {
            Self::Float(f64::from(self) * f64::from(other))
        }
    }
}

impl Div for VariableValue {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Self::Integer(a / b)
        } else {
            Self::Float(f64::from(self) / f64::from(other))
        }
    }
}

impl PartialOrd for VariableValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.is_number(), other.is_number()) {
            (true, true) => {
                let a = f64::from(self);
                let b = f64::from(other);
                a.partial_cmp(&b)
            }
            _ => match (self, other) {
                (Self::Bool(a), Self::Bool(b)) => {
                    if a == b {
                        Some(std::cmp::Ordering::Equal)
                    } else {
                        Some(std::cmp::Ordering::Less)
                    }
                }
                _ => None,
            },
        }
    }
}

impl BitOr for VariableValue {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        Self::Bool(bool::from(self) | bool::from(other))
    }
}

impl BitAnd for VariableValue {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        Self::Bool(bool::from(self) & bool::from(other))
    }
}

impl Not for VariableValue {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Bool(!bool::from(self))
    }
}
