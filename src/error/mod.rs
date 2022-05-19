#[allow(clippy::module_name_repetitions)]
pub mod error_kind;

use core::fmt;

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
    // TODO: Maybe fix this later
    #[allow(clippy::needless_pass_by_value)]
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
}

pub type Result<'a, T> = std::result::Result<T, RaoulError<'a>>;
pub type Results<'a, T> = std::result::Result<T, Vec<RaoulError<'a>>>;
