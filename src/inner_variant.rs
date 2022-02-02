// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionVariant {
    Simple(SimpleExpression),
    Compound(CompoundExpression),
    With(WithExpression),
}

impl ExpressionVariant {
    pub const fn simple(
        identifier: String,
        document_ref: Option<String>,
        license_ref: bool,
    ) -> Self {
        Self::Simple(SimpleExpression::new(identifier, document_ref, license_ref))
    }

    pub fn and(left: Self, right: Self) -> Self {
        Self::Compound(CompoundExpression::new(left, SpdxOperator::And, right))
    }

    pub fn or(left: Self, right: Self) -> Self {
        Self::Compound(CompoundExpression::new(left, SpdxOperator::Or, right))
    }

    pub const fn with(license: SimpleExpression, exception: String) -> Self {
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
    pub const fn new(identifier: String, document_ref: Option<String>, license_ref: bool) -> Self {
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
    pub left: Box<ExpressionVariant>,
    pub operator: SpdxOperator,
    pub right: Box<ExpressionVariant>,
}

impl CompoundExpression {
    pub(crate) fn new(
        left: ExpressionVariant,
        operator: SpdxOperator,
        right: ExpressionVariant,
    ) -> Self {
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
    pub const fn new(license: SimpleExpression, exception: String) -> Self {
        Self { license, exception }
    }
}
