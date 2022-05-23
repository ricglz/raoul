#[allow(clippy::module_name_repetitions)]
pub mod error_kind;

use core::fmt;
use std::fmt::Debug;

use pest::error::{Error, ErrorVariant};
use pest::Span;

use crate::ast::AstNode;
use crate::parser::Rule;

use self::error_kind::RaoulErrorKind;

#[derive(PartialEq, Eq, Clone)]
#[allow(clippy::module_name_repetitions)]
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
    pub fn new<'a>(node: &AstNode<'a>, kind: RaoulErrorKind) -> RaoulError<'a> {
        RaoulError {
            kind,
            span: node.span.clone(),
        }
    }

    pub fn new_vec<'a>(node: &AstNode<'a>, kind: RaoulErrorKind) -> Vec<RaoulError<'a>> {
        vec![RaoulError::new(node, kind)]
    }

    fn from_results_iter<'a, T, I: IntoIterator<Item = Results<'a, T>>>(
        iter: I,
    ) -> Vec<RaoulError<'a>> {
        iter.into_iter()
            .filter_map(Results::err)
            .flatten()
            .collect()
    }

    pub fn create_results<'a, T, I: IntoIterator<Item = Results<'a, T>>>(
        iter: I,
    ) -> Results<'a, ()> {
        let errors = RaoulError::from_results_iter(iter);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn create_partition<'a, T: Debug, I: IntoIterator<Item = Results<'a, T>>>(
        iter: I,
    ) -> Results<'a, Vec<T>> {
        let (oks, errors): (Vec<_>, Vec<_>) = iter.into_iter().partition(Results::is_ok);
        if errors.is_empty() {
            Ok(oks.into_iter().map(Results::unwrap).collect())
        } else {
            Err(errors.into_iter().flat_map(Results::unwrap_err).collect())
        }
    }
}

pub type Result<'a, T> = std::result::Result<T, RaoulError<'a>>;
pub type Results<'a, T> = std::result::Result<T, Vec<RaoulError<'a>>>;
