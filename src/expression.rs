#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpdxExpression {
    Simple(SimpleExpression),
    Compound(CompoundExpression),
    With(WithExpression),
}

impl SpdxExpression {
    pub(crate) fn simple(
        identifier: String,
        document_ref: Option<String>,
        license_ref: bool,
    ) -> Self {
        Self::Simple(SimpleExpression::new(identifier, document_ref, license_ref))
    }

    pub(crate) fn and(left: Self, right: Self) -> Self {
        Self::Compound(CompoundExpression::new(left, SpdxOperator::And, right))
    }

    pub(crate) fn or(left: Self, right: Self) -> Self {
        Self::Compound(CompoundExpression::new(left, SpdxOperator::Or, right))
    }

    pub(crate) fn with(license: SimpleExpression, exception: String) -> Self {
        Self::With(WithExpression::new(license, exception))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleExpression {
    pub identifier: String,
    pub document_ref: Option<String>,
    pub license_ref: bool,
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
pub enum SpdxOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundExpression {
    pub left: Box<SpdxExpression>,
    pub operator: SpdxOperator,
    pub right: Box<SpdxExpression>,
}

impl CompoundExpression {
    pub(crate) fn new(left: SpdxExpression, operator: SpdxOperator, right: SpdxExpression) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithExpression {
    pub license: SimpleExpression,
    pub exception: String,
}

impl WithExpression {
    pub(crate) fn new(license: SimpleExpression, exception: String) -> Self {
        Self { license, exception }
    }
}
