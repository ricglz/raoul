use std::fmt;
use std::io::stdin;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Not, Sub};

use crate::vm::VMResult;
use crate::{ast::ast_kind::AstNodeKind, enums::Types};

#[derive(Clone, PartialEq)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl VariableValue {
    pub fn from_stdin() -> Self {
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        Self::String(line)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Float(_) | Self::String(_))
    }

    pub fn is_boolish(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Bool(_))
    }

    #[inline]
    fn cast_to_bool(&self) -> VariableValue {
        Self::Bool(bool::from(self))
    }

    #[inline]
    fn cast_to_float(&self) -> VMResult<VariableValue> {
        Ok(Self::Float(f64::try_from(self)?))
    }

    #[inline]
    fn cast_to_int(&self) -> VMResult<VariableValue> {
        Ok(Self::Integer(i64::try_from(self)?))
    }

    pub fn cast_to(&self, to: Types) -> VMResult<VariableValue> {
        match to {
            Types::Bool => Ok(self.cast_to_bool()),
            Types::Float => self.cast_to_float(),
            Types::Int => self.cast_to_int(),
            _ => Ok(self.clone()),
        }
    }

    pub fn increase(&self) -> VMResult<Self> {
        match self {
            Self::Integer(v) => Ok(Self::Integer(v + 1)),
            v => {
                if v.is_number() {
                    self.cast_to_float()? + Self::Float(1.0)
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl From<&VariableValue> for Types {
    fn from(v: &VariableValue) -> Self {
        match v {
            VariableValue::Integer(_) => Types::Int,
            VariableValue::Float(_) => Types::Float,
            VariableValue::String(_) => Types::String,
            VariableValue::Bool(_) => Types::Bool,
        }
    }
}

impl From<AstNodeKind<'_>> for VariableValue {
    fn from(v: AstNodeKind) -> Self {
        match v {
            AstNodeKind::Integer(value) => VariableValue::Integer(value),
            AstNodeKind::Float(value) => VariableValue::Float(value),
            AstNodeKind::String(value) => VariableValue::String(value),
            AstNodeKind::Bool(value) => VariableValue::Bool(value),
            _ => unreachable!(),
        }
    }
}

impl TryFrom<VariableValue> for f64 {
    type Error = &'static str;

    fn try_from(v: VariableValue) -> VMResult<Self> {
        if let VariableValue::Float(a) = &v {
            return Ok(*a);
        }
        let string = match v {
            VariableValue::Integer(a) => a.to_string(),
            VariableValue::String(a) => a,
            _ => unreachable!(),
        };
        match string.parse::<Self>() {
            Ok(a) => Ok(a),
            Err(_) => Err("Could not parse to float"),
        }
    }
}

impl TryFrom<&VariableValue> for f64 {
    type Error = &'static str;

    fn try_from(v: &VariableValue) -> VMResult<Self> {
        Self::try_from(v.clone())
    }
}

impl From<f64> for VariableValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
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
        Self::from(v.clone())
    }
}

impl From<usize> for VariableValue {
    fn from(v: usize) -> Self {
        Self::Integer(v.try_into().unwrap())
    }
}

impl From<VariableValue> for usize {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::Integer(v) => v.try_into().unwrap(),
            _ => unreachable!(),
        }
    }
}

impl From<VariableValue> for String {
    fn from(v: VariableValue) -> Self {
        match v {
            VariableValue::String(v) => v,
            _ => unreachable!(),
        }
    }
}

impl TryFrom<VariableValue> for i64 {
    type Error = &'static str;

    fn try_from(v: VariableValue) -> VMResult<Self> {
        if let VariableValue::Integer(a) = &v {
            return Ok(*a);
        }
        if let VariableValue::Bool(a) = &v {
            return match a {
                true => Ok(1),
                false => Ok(0),
            };
        }
        let string = match v {
            VariableValue::Float(a) => a.floor().to_string(),
            VariableValue::String(a) => a,
            _ => unreachable!(),
        };
        match string.parse::<Self>() {
            Ok(a) => Ok(a),
            Err(_) => Err("Could not parse to float"),
        }
    }
}

impl TryFrom<&VariableValue> for i64 {
    type Error = &'static str;

    fn try_from(v: &VariableValue) -> VMResult<Self> {
        Self::try_from(v.clone())
    }
}

impl fmt::Debug for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            VariableValue::Bool(value) => value.to_string(),
            VariableValue::Integer(value) => value.to_string(),
            VariableValue::Float(value) => value.to_string(),
            VariableValue::String(value) => value.clone(),
        };
        write!(f, "{}", value)
    }
}

impl Add for VariableValue {
    type Output = VMResult<Self>;

    fn add(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Ok(Self::Integer(a + b))
        } else {
            Ok(Self::Float(f64::try_from(self)? + f64::try_from(other)?))
        }
    }
}

impl Sub for VariableValue {
    type Output = VMResult<Self>;

    fn sub(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Ok(Self::Integer(a - b))
        } else {
            Ok(Self::Float(f64::try_from(self)? - f64::try_from(other)?))
        }
    }
}

impl Mul for VariableValue {
    type Output = VMResult<Self>;

    fn mul(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            Ok(Self::Integer(a * b))
        } else {
            Ok(Self::Float(f64::try_from(self)? * f64::try_from(other)?))
        }
    }
}

impl Div for VariableValue {
    type Output = VMResult<Self>;

    fn div(self, other: Self) -> Self::Output {
        if let (Self::Integer(a), Self::Integer(b)) = (self.clone(), other.clone()) {
            match b {
                0 => Err("Attempt to divide by zero"),
                b => Ok(Self::Integer(a / b)),
            }
        } else {
            match (f64::try_from(self)?, f64::try_from(other)?) {
                (_, b) if b == 0.0 => Err("Attempt to divide by zero"),
                (a, b) => Ok(Self::Float(a / b)),
            }
        }
    }
}

impl PartialOrd for VariableValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.is_number(), other.is_number()) {
            (true, true) => match (f64::try_from(self), f64::try_from(other)) {
                (Ok(a), Ok(b)) => a.partial_cmp(&b),
                _ => None,
            },
            _ => match (self.is_boolish(), other.is_boolish()) {
                (true, true) => bool::from(self).partial_cmp(&bool::from(other)),
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
