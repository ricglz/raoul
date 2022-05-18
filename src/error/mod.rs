pub mod error_kind;

use core::fmt;

use pest::error::{Error, ErrorVariant};
use pest::Span;

use crate::ast::AstNode;
use crate::parser::Rule;

use self::error_kind::RaoulErrorKind;

#[derive(PartialEq, Eq, Clone)]
pub struct RaoulError<'a> {
    kind: RaoulErrorKind,
    span: Span<'a>,
}

impl fmt::Debug for RaoulError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = format!("{:?}", self.kind);
        let error: Error<Rule> =
            Error::new_from_span(ErrorVariant::CustomError { message }, self.span.clone());
        write!(f, "{}", error)
    }
}

impl RaoulError<'_> {
    pub fn new(node: AstNode, kind: RaoulErrorKind) -> RaoulError {
        RaoulError {
            kind,
            span: node.span.clone(),
        }
    }

    pub fn new_vec(node: AstNode, kind: RaoulErrorKind) -> Vec<RaoulError> {
        vec![RaoulError::new(node, kind)]
    }

    pub fn is_invalid(&self) -> bool {
        self.kind == RaoulErrorKind::Invalid
    }
}

pub type Result<'a, T> = std::result::Result<T, RaoulError<'a>>;
pub type Results<'a, T> = std::result::Result<T, Vec<RaoulError<'a>>>;
