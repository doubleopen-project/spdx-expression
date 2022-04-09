// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

//! Private inner structs for [`crate::SPDXExpression`].

use std::fmt::Display;

use nom::Finish;

use crate::{error::SpdxExpressionError, parser::expr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleExpression {
    pub identifier: String,
    pub document_ref: Option<String>,
    pub license_ref: bool,
}

impl Display for SimpleExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let document_ref = match &self.document_ref {
            Some(document_ref) => {
                format!("DocumentRef-{}:", document_ref)
            }
            None => "".to_string(),
        };

        let license_ref = if self.license_ref { "LicenseRef-" } else { "" };
        write!(
            f,
            "{document_ref}{license_ref}{identifier}",
            identifier = self.identifier
        )
    }
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
pub struct WithExpression {
    pub license: SimpleExpression,
    pub exception: String,
}

impl WithExpression {
    pub const fn new(license: SimpleExpression, exception: String) -> Self {
        Self { license, exception }
    }
}

impl Display for WithExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{license} WITH {exception}",
            license = self.license,
            exception = self.exception
        )
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum ExpressionVariant {
    Simple(SimpleExpression),
    With(WithExpression),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Parens(Box<Self>),
}

impl Display for ExpressionVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::ExpressionVariant::{And, Or, Parens, Simple, With};

        match self {
            Simple(expression) => write!(f, "{expression}"),
            With(expression) => write!(f, "{expression}"),
            And(left, right) => write!(f, "{left} AND {right}"),
            Or(left, right) => write!(f, "{left} OR {right}"),
            Parens(expression) => write!(f, "({expression})"),
        }
    }
}

impl ExpressionVariant {
    pub fn parse(i: &str) -> Result<Self, SpdxExpressionError> {
        let (remaining, expression) = expr(i)
            .finish()
            .map_err(|_| SpdxExpressionError::Parse(i.to_string()))?;

        if remaining.is_empty() {
            Ok(expression)
        } else {
            Err(SpdxExpressionError::Parse(i.to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_simple_correctly() {
        let expression =
            ExpressionVariant::Simple(SimpleExpression::new("MIT".to_string(), None, false));
        assert_eq!(expression.to_string(), "MIT".to_string());
    }

    #[test]
    fn display_licenseref_correctly() {
        let expression =
            ExpressionVariant::Simple(SimpleExpression::new("license".to_string(), None, true));
        assert_eq!(expression.to_string(), "LicenseRef-license".to_string());
    }

    #[test]
    fn display_documentref_correctly() {
        let expression = ExpressionVariant::Simple(SimpleExpression::new(
            "license".to_string(),
            Some("document".to_string()),
            true,
        ));
        assert_eq!(
            expression.to_string(),
            "DocumentRef-document:LicenseRef-license".to_string()
        );
    }

    #[test]
    fn display_with_expression_correctly() {
        let expression = ExpressionVariant::With(WithExpression::new(
            SimpleExpression::new("license".to_string(), None, false),
            "exception".to_string(),
        ));
        assert_eq!(expression.to_string(), "license WITH exception".to_string());
    }

    #[test]
    fn display_and_expression_correctly() {
        let expression = ExpressionVariant::And(
            Box::new(ExpressionVariant::And(
                Box::new(ExpressionVariant::Simple(SimpleExpression::new(
                    "license1".to_string(),
                    None,
                    false,
                ))),
                Box::new(ExpressionVariant::Simple(SimpleExpression::new(
                    "license2".to_string(),
                    None,
                    false,
                ))),
            )),
            Box::new(ExpressionVariant::Simple(SimpleExpression::new(
                "license3".to_string(),
                None,
                false,
            ))),
        );
        assert_eq!(
            expression.to_string(),
            "license1 AND license2 AND license3".to_string()
        );
    }

    #[test]
    fn display_or_expression_correctly() {
        let expression = ExpressionVariant::Or(
            Box::new(ExpressionVariant::Or(
                Box::new(ExpressionVariant::Simple(SimpleExpression::new(
                    "license1".to_string(),
                    None,
                    false,
                ))),
                Box::new(ExpressionVariant::Simple(SimpleExpression::new(
                    "license2".to_string(),
                    None,
                    false,
                ))),
            )),
            Box::new(ExpressionVariant::Simple(SimpleExpression::new(
                "license3".to_string(),
                None,
                false,
            ))),
        );
        assert_eq!(
            expression.to_string(),
            "license1 OR license2 OR license3".to_string()
        );
    }
}
