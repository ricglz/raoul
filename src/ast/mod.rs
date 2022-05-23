#[allow(clippy::module_name_repetitions)]
pub mod ast_kind;

use crate::dir_func::variable::Dimensions;

use self::ast_kind::AstNodeKind;
use pest::Span;
use std::fmt;

#[derive(PartialEq, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct AstNode<'a> {
    pub kind: AstNodeKind<'a>,
    pub span: Span<'a>,
}

impl<'a> From<&AstNode<'a>> for String {
    fn from(val: &AstNode) -> Self {
        Self::from(&val.kind)
    }
}

impl<'a> From<AstNode<'a>> for String {
    fn from(val: AstNode) -> Self {
        Self::from(&val)
    }
}

impl From<Box<AstNode<'_>>> for String {
    fn from(val: Box<AstNode>) -> Self {
        String::from(*val)
    }
}

impl From<&Box<AstNode<'_>>> for String {
    fn from(val: &Box<AstNode>) -> Self {
        String::from(*val.clone())
    }
}

impl<'a> From<AstNode<'a>> for usize {
    fn from(val: AstNode) -> Self {
        val.kind.into()
    }
}

impl<'a> AstNode<'a> {
    pub fn expand_node(v: &AstNode<'a>) -> Vec<AstNode<'a>> {
        match &v.kind {
            AstNodeKind::Decision { statements, .. }
            | AstNodeKind::ElseBlock { statements }
            | AstNodeKind::While { statements, .. } => {
                statements.iter().flat_map(AstNode::expand_node).collect()
            }
            AstNodeKind::For {
                statements,
                assignment,
                ..
            } => vec![*assignment.clone()]
                .iter()
                .chain(statements)
                .flat_map(AstNode::expand_node)
                .collect(),
            _ => vec![v.clone()],
        }
    }

    pub fn expand_array(&self) -> &Vec<AstNode<'a>> {
        match &self.kind {
            AstNodeKind::Array(exprs) => exprs,
            _ => unreachable!(),
        }
    }

    pub fn new(kind: AstNodeKind<'a>, span: &Span<'a>) -> AstNode<'a> {
        AstNode {
            kind,
            span: span.clone(),
        }
    }

    pub fn is_array(&self) -> bool {
        self.kind.is_array()
    }

    pub fn is_declaration(&self) -> bool {
        self.kind.is_declaration()
    }

    pub fn get_dimensions(&self) -> Result<Dimensions, Dimensions> {
        self.kind.get_dimensions()
    }
}

impl fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
