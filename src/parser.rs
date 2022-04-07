use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::{
        complete::{multispace0, multispace1},
        streaming::char,
    },
    combinator::{complete, map, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair},
    AsChar, Finish, IResult,
};

use crate::{
    error::SpdxExpressionError,
    inner_variant::{SimpleExpression, WithExpression},
};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Expression {
    Simple(SimpleExpression),
    With(WithExpression),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Parens(Box<Self>),
}

impl Expression {
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

#[derive(Debug)]
enum Oper {
    And,
    Or,
}

fn parens(i: &str) -> IResult<&str, Expression> {
    delimited(
        multispace0,
        delimited(
            tag("("),
            map(expr, |e| Expression::Parens(Box::new(e))),
            tag(")"),
        ),
        multispace0,
    )(i)
}

fn factor(i: &str) -> IResult<&str, Expression> {
    alt((
        delimited(multispace0, with_expression, multispace0),
        map(
            delimited(multispace0, simple_license_expression, multispace0),
            Expression::Simple,
        ),
        parens,
    ))(i)
}

fn with_expression(i: &str) -> IResult<&str, Expression> {
    map(
        separated_pair(
            simple_license_expression,
            delimited(multispace1, tag_no_case("WITH"), multispace1),
            idstring,
        ),
        |(lic, exc)| Expression::With(WithExpression::new(lic, exc.to_string())),
    )(i)
}

fn fold_exprs(initial: Expression, remainder: Vec<(Oper, Expression)>) -> Expression {
    remainder.into_iter().fold(initial, |acc, pair| {
        let (oper, expr) = pair;
        match oper {
            Oper::And => Expression::And(Box::new(acc), Box::new(expr)),
            Oper::Or => Expression::Or(Box::new(acc), Box::new(expr)),
        }
    })
}

fn term(i: &str) -> IResult<&str, Expression> {
    let (i, initial) = factor(i)?;
    let (i, remainder) = many0(|i| {
        let (i, and) = preceded(tag_no_case("AND"), factor)(i)?;
        Ok((i, (Oper::And, and)))
    })(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

fn expr(i: &str) -> IResult<&str, Expression> {
    let (i, initial) = term(i)?;
    let (i, remainder) = many0(|i| {
        let (i, or) = preceded(tag_no_case("OR"), term)(i)?;
        Ok((i, (Oper::Or, or)))
    })(i)?;

    Ok((i, fold_exprs(initial, remainder)))
}

fn idstring(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanum() || c == '-' || c == '.')(i)
}

fn license_idstring(i: &str) -> IResult<&str, &str> {
    recognize(pair(idstring, opt(complete(char('+')))))(i)
}

fn document_ref(i: &str) -> IResult<&str, &str> {
    delimited(tag("DocumentRef-"), idstring, char(':'))(i)
}

fn license_ref(i: &str) -> IResult<&str, (Option<&str>, &str)> {
    separated_pair(opt(document_ref), tag("LicenseRef-"), idstring)(i)
}

fn simple_license_expression(i: &str) -> IResult<&str, SimpleExpression> {
    alt((
        map(license_ref, |(document_ref, id)| {
            let document_ref = document_ref.map(std::string::ToString::to_string);
            SimpleExpression::new(id.to_string(), document_ref, true)
        }),
        map(license_idstring, |id| {
            SimpleExpression::new(id.to_string(), None, false)
        }),
    ))(i)
}

#[cfg(test)]
mod test_parser {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_a_license_id_correctly() {
        let parsed = Expression::parse("spdx.license-id").unwrap();
        assert_eq!(
            parsed,
            Expression::Simple(SimpleExpression::new(
                "spdx.license-id".to_string(),
                None,
                false
            ))
        );
    }

    #[test]
    fn test_parse_a_license_id_starting_with_a_digit_correctly() {
        let parsed = Expression::parse("0license").unwrap();
        assert_eq!(
            parsed,
            Expression::Simple(SimpleExpression::new("0license".to_string(), None, false))
        );
    }

    #[test]
    fn test_parse_a_license_id_with_any_later_version_correctly() {
        let parsed = Expression::parse("license+").unwrap();
        assert_eq!(
            parsed,
            Expression::Simple(SimpleExpression::new("license+".to_string(), None, false))
        );
    }

    #[test]
    fn test_parse_a_document_ref_correctly() {
        let parsed = Expression::parse("DocumentRef-document:LicenseRef-license").unwrap();
        assert_eq!(
            parsed,
            Expression::Simple(SimpleExpression::new(
                "license".to_string(),
                Some("document".to_string()),
                true
            ))
        );
    }

    #[test]
    fn test_parse_a_license_ref_correctly() {
        let parsed = Expression::parse("LicenseRef-license").unwrap();
        assert_eq!(
            parsed,
            Expression::Simple(SimpleExpression::new("license".to_string(), None, true))
        );
    }

    #[test]
    fn test_parse_a_with_expression_correctly() {
        let parsed = Expression::parse("license WITH exception").unwrap();
        assert_eq!(
            parsed,
            Expression::With(WithExpression::new(
                SimpleExpression::new("license".to_string(), None, false),
                "exception".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_a_complex_expression_correctly() {
        let parsed = Expression::parse(
            "license1+ and ((license2 with exception1) OR license3+ AND license4 WITH exception2)",
        )
        .unwrap();

        assert_eq!(
            parsed,
            Expression::And(
                Box::new(Expression::Simple(SimpleExpression::new(
                    "license1+".to_string(),
                    None,
                    false
                ))),
                Box::new(Expression::Parens(Box::new(Expression::Or(
                    Box::new(Expression::Parens(Box::new(Expression::With(
                        WithExpression::new(
                            SimpleExpression::new("license2".to_string(), None, false),
                            "exception1".to_string()
                        )
                    )))),
                    Box::new(Expression::And(
                        Box::new(Expression::Simple(SimpleExpression::new(
                            "license3+".to_string(),
                            None,
                            false
                        ))),
                        Box::new(Expression::With(WithExpression::new(
                            SimpleExpression::new("license4".to_string(), None, false),
                            "exception2".to_string()
                        )))
                    )),
                ))))
            )
        );
    }

    #[test]
    fn test_bind_plus_stronger_than_with() {
        let parsed = Expression::parse("license+ WITH exception").unwrap();
        assert_eq!(
            parsed,
            Expression::With(WithExpression::new(
                SimpleExpression::new("license+".to_string(), None, false),
                "exception".to_string()
            ))
        );
    }

    #[test]
    fn test_bind_with_stronger_than_and() {
        let parsed = Expression::parse("license1 AND license2 WITH exception").unwrap();
        assert_eq!(
            parsed,
            Expression::And(
                Box::new(Expression::Simple(SimpleExpression::new(
                    "license1".to_string(),
                    None,
                    false
                ))),
                Box::new(Expression::With(WithExpression::new(
                    SimpleExpression::new("license2".to_string(), None, false),
                    "exception".to_string()
                )))
            )
        );
    }

    #[test]
    fn test_bind_and_stronger_than_or() {
        let parsed = Expression::parse("license1 OR license2 AND license3").unwrap();
        assert_eq!(
            parsed,
            Expression::Or(
                Box::new(Expression::Simple(SimpleExpression::new(
                    "license1".to_string(),
                    None,
                    false
                ))),
                Box::new(Expression::And(
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license2".to_string(),
                        None,
                        false
                    ))),
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license3".to_string(),
                        None,
                        false
                    )))
                ))
            )
        );
    }

    #[test]
    fn test_the_and_operator_left_associative() {
        let parsed = Expression::parse("license1 AND license2 AND license3").unwrap();
        assert_eq!(
            parsed,
            Expression::And(
                Box::new(Expression::And(
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license1".to_string(),
                        None,
                        false
                    ))),
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license2".to_string(),
                        None,
                        false
                    )))
                )),
                Box::new(Expression::Simple(SimpleExpression::new(
                    "license3".to_string(),
                    None,
                    false
                ))),
            )
        );
    }

    #[test]
    fn test_the_or_operator_left_associative() {
        let parsed = Expression::parse("license1 OR license2 OR license3").unwrap();
        assert_eq!(
            parsed,
            Expression::Or(
                Box::new(Expression::Or(
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license1".to_string(),
                        None,
                        false
                    ))),
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license2".to_string(),
                        None,
                        false
                    )))
                )),
                Box::new(Expression::Simple(SimpleExpression::new(
                    "license3".to_string(),
                    None,
                    false
                ))),
            )
        );
    }

    #[test]
    fn test_parentheses_for_binding_strength_of_operators() {
        let parsed = Expression::parse("(license1 OR license2) AND license3").unwrap();
        assert_eq!(
            parsed,
            Expression::And(
                Box::new(Expression::Parens(Box::new(Expression::Or(
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license1".to_string(),
                        None,
                        false
                    ))),
                    Box::new(Expression::Simple(SimpleExpression::new(
                        "license2".to_string(),
                        None,
                        false
                    )))
                )))),
                Box::new(Expression::Simple(SimpleExpression::new(
                    "license3".to_string(),
                    None,
                    false
                ))),
            )
        );
    }

    #[test]
    fn test_fail_if_plus_is_used_in_an_exception_expression() {
        let parsed = Expression::parse("license WITH exception+");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_fail_if_a_compound_expressions_is_used_before_with() {
        let parsed = Expression::parse("(license1 AND license2) WITH exception");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_fail_on_an_invalid_symbol() {
        let parsed = Expression::parse("/");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_fail_on_a_syntax_error() {
        let parsed = Expression::parse("((");
        assert!(parsed.is_err());
    }
}
