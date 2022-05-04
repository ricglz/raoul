pub mod ast_kind;

use crate::dir_func::variable::Dimensions;

use self::ast_kind::AstNodeKind;
use pest::Span;
use std::fmt;

#[derive(PartialEq, Clone)]
pub struct AstNode<'a> {
    pub kind: AstNodeKind<'a>,
    pub span: Span<'a>,
}

impl<'a> From<AstNode<'a>> for String {
    fn from(val: AstNode) -> Self {
        val.kind.into()
    }
}

impl<'a> From<AstNode<'a>> for usize {
    fn from(val: AstNode) -> Self {
        val.kind.into()
    }
}

impl<'a> AstNode<'a> {
    pub fn expand_node(v: AstNode<'a>) -> Vec<AstNode<'a>> {
        let node = v.clone();
        match &v.kind {
            AstNodeKind::Decision { statements, .. }
            | AstNodeKind::ElseBlock { statements }
            | AstNodeKind::While { statements, .. } => statements
                .to_owned()
                .into_iter()
                .flat_map(AstNode::expand_node)
                .collect(),
            AstNodeKind::For {
                statements,
                assignment,
                ..
            } => vec![*assignment.clone()]
                .to_owned()
                .into_iter()
                .chain(statements.to_owned())
                .flat_map(AstNode::expand_node)
                .collect(),
            _ => vec![node],
        }
    }

    pub fn new(kind: AstNodeKind<'a>, span: Span<'a>) -> AstNode<'a> {
        AstNode { kind, span }
    }

    pub fn is_array(&self) -> bool {
        self.kind.is_array()
    }

    pub fn get_dimensions(&self) -> Dimensions {
        self.kind.get_dimensions()
    }
}

impl fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}
