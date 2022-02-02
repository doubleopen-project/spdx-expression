// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

use std::fmt::Display;

use crate::{error::SpdxExpressionError, inner_variant::ExpressionVariant, parse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPDXExpression {
    /// Store the original input string for easier display.
    expression_string: String,

    /// The parsed expression.
    inner: ExpressionVariant,
}

impl SPDXExpression {
    /// Parse `Self` from a string. The input expression needs to be a syntactically valid SPDX
    /// expression, `NONE` or `NOASSERTION`. The parser accepts license identifiers that are not
    /// valid SPDX.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spdx_expression::expression::SPDXExpression;
    /// # use spdx_expression::error::SpdxExpressionError;
    /// #
    /// let expression = SPDXExpression::parse("MIT")?;
    /// # Ok::<(), SpdxExpressionError>(())
    /// ```
    ///
    /// License expressions need to be syntactically valid, but they can include license
    /// identifiers not on the SPDX license list or not specified with `LicenseRef`.
    ///
    /// ```
    /// # use spdx_expression::expression::SPDXExpression;
    /// # use spdx_expression::error::SpdxExpressionError;
    /// #
    /// let expression = SPDXExpression::parse("MIT OR InvalidLicenseId")?;
    /// # Ok::<(), SpdxExpressionError>(())
    /// ```
    pub fn parse(expression: &str) -> Result<Self, SpdxExpressionError> {
        Ok(Self {
            expression_string: expression.to_owned(),
            inner: parse::spdx_expression(expression).unwrap().1,
        })
    }

    /// Get all license and exception identifiers from the `SPDXExpression`. Returns the licenses
    /// alphabetically sorted and deduped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spdx_expression::expression::SPDXExpression;
    /// # use spdx_expression::error::SpdxExpressionError;
    /// #
    /// let expression = SPDXExpression::parse("MIT OR Apache-2.0")?;
    /// let licenses = expression.licenses();
    /// assert_eq!(licenses, vec!["Apache-2.0".to_string(), "MIT".to_string()]);
    /// # Ok::<(), SpdxExpressionError>(())
    /// ```
    pub fn licenses(&self) -> Vec<String> {
        let licenses = self.expression_string.split_ascii_whitespace();
        let licenses = licenses.filter(|&i| i != "OR" && i != "AND" && i != "WITH");
        let licenses = licenses.map(|i| i.replace("(", "").replace(")", ""));
        let mut licenses = licenses.collect::<Vec<_>>();
        licenses.sort_unstable();
        licenses.dedup();
        licenses
    }
}

impl Display for SPDXExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expression_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        let expression = SPDXExpression::parse("MIT").unwrap();
        assert_eq!(expression.to_string(), "MIT");
    }

    #[test]
    fn test_parse_compound_or() {
        let expression = SPDXExpression::parse("MIT OR Apache-2.0").unwrap();
        assert_eq!(expression.to_string(), "MIT OR Apache-2.0");
    }

    #[test]
    fn test_licenses_from_simple_expression() {
        let expression = SPDXExpression::parse("MIT").unwrap();
        let licenses = expression.licenses();
        assert_eq!(licenses, vec!["MIT".to_string()]);
    }

    #[test]
    fn test_licenses_from_compound_or_expression() {
        let expression = SPDXExpression::parse("MIT OR Apache-2.0").unwrap();
        let licenses = expression.licenses();
        assert_eq!(licenses, vec!["Apache-2.0".to_string(), "MIT".to_string()]);
    }

    #[test]
    fn test_licenses_from_compound_parentheses_expression() {
        let expression = SPDXExpression::parse(
            "(MIT OR Apache-2.0 AND (GPL-2.0-only WITH Classpath-exception-2.0 OR ISC))",
        )
        .unwrap();
        let licenses = expression.licenses();
        assert_eq!(
            licenses,
            vec![
                "Apache-2.0".to_string(),
                "Classpath-exception-2.0".to_string(),
                "GPL-2.0-only".to_string(),
                "ISC".to_string(),
                "MIT".to_string()
            ]
        );
    }
}
