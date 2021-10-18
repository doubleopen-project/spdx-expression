#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpdxExpression {
    SimpleExpression(SimpleExpression),
    CompoundExpression(CompoundExpression),
    WithExpression(WithExpression),
}

impl SpdxExpression {
    pub(crate) fn simple(
        identifier: String,
        document_ref: Option<String>,
        license_ref: bool,
    ) -> Self {
        Self::SimpleExpression(SimpleExpression::new(identifier, document_ref, license_ref))
    }

    pub(crate) fn and(left: Self, right: Self) -> Self {
        Self::CompoundExpression(CompoundExpression::new(left, SpdxOperator::And, right))
    }

    pub(crate) fn or(left: Self, right: Self) -> Self {
        Self::CompoundExpression(CompoundExpression::new(left, SpdxOperator::Or, right))
    }

    pub(crate) fn with(license: SimpleExpression, exception: String) -> Self {
        Self::WithExpression(WithExpression::new(license, exception))
    }
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
pub(crate) struct WithExpression {
    pub(crate) license: SimpleExpression,
    pub(crate) exception: String,
}

impl WithExpression {
    pub(crate) fn new(license: SimpleExpression, exception: String) -> Self {
        Self { license, exception }
    }
}
