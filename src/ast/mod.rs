mod ast_kind;

use std::fmt;

use pest::Span;

use crate::enums::{Operations, Types};
use crate::parser::Statements;

use self::ast_kind::AstNodeKind;

#[derive(PartialEq, Clone)]
pub struct AstNode<'a> {
    kind: AstNodeKind<'a>,
    span: Span<'a>,
}

impl<'a> From<AstNode<'a>> for String {
    fn from(val: AstNode) -> Self {
        val.kind.into()
    }
}

impl fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
