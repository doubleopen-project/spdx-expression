#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpdxExpression {
    SimpleExpression(SimpleExpression),
    CompoundExpresion(CompoundExpression),
    WithExpression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimpleExpression {
    pub(crate) identifier: String,
    pub(crate) document_ref: Option<String>,
    pub(crate) license_ref: bool,
}

impl SimpleExpression {
    pub(crate) fn new(identifier: String, document_ref: Option<String>, license_ref: bool) -> Self {
        Self {
            identifier,
            document_ref,
            license_ref,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpdxOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompoundExpression {
    pub(crate) left: Box<SpdxExpression>,
    pub(crate) operator: SpdxOperator,
    pub(crate) right: Box<SpdxExpression>,
}
