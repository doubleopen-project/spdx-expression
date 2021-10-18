use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::{
        complete::{multispace0, space1},
        streaming::char,
    },
    combinator::{complete, map, opt, recognize},
    error::VerboseError,
    sequence::{delimited, pair, separated_pair, tuple},
    AsChar, IResult,
};

use crate::expression::{SimpleExpression, SpdxExpression};

pub fn spdx_expression(i: &str) -> IResult<&str, SpdxExpression, VerboseError<&str>> {
    alt((
        ws(parentheses),
        ws(and_expression),
        ws(or_expression),
        ws(simple_expression),
    ))(i)
}

fn idstring(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(|c: char| c.is_alphanum() || c == '-' || c == '.')(i)
}

fn license_idstring(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    recognize(pair(idstring, opt(complete(char('+')))))(i)
}

fn document_ref(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(tag("DocumentRef-"), idstring, char(':'))(i)
}

fn license_ref(i: &str) -> IResult<&str, (Option<&str>, &str), VerboseError<&str>> {
    separated_pair(opt(document_ref), tag("LicenseRef-"), idstring)(i)
}

fn simple_license_expression(i: &str) -> IResult<&str, SimpleExpression, VerboseError<&str>> {
    alt((
        map(license_ref, |(document_ref, id)| {
            let document_ref = document_ref.map(|document_ref| document_ref.to_string());
            SimpleExpression::new(id.to_string(), document_ref, true)
        }),
        map(license_idstring, |id| {
            SimpleExpression::new(id.to_string(), None, false)
        }),
    ))(i)
}

fn simple_expression(i: &str) -> IResult<&str, SpdxExpression, VerboseError<&str>> {
    alt((
        map(license_ref, |(document_ref, id)| {
            let document_ref = document_ref.map(|document_ref| document_ref.to_string());
            SpdxExpression::simple(id.to_string(), document_ref, true)
        }),
        map(license_idstring, |id| {
            SpdxExpression::simple(id.to_string(), None, false)
        }),
    ))(i)
}

fn with_expression(i: &str) -> IResult<&str, SpdxExpression, VerboseError<&str>> {
    map(
        tuple((simple_license_expression, space1, with, space1, idstring)),
        |(license, _, _, _, exception)| SpdxExpression::with(license, exception.to_string()),
    )(i)
}

fn and_expression(i: &str) -> IResult<&str, SpdxExpression, VerboseError<&str>> {
    map(
        tuple((
            alt((parentheses, simple_expression)),
            ws(and),
            alt((
                ws(parentheses),
                ws(and_expression),
                ws(or_expression),
                ws(with_expression),
                ws(simple_expression),
            )),
        )),
        |(left, _, right)| SpdxExpression::and(left, right),
    )(i)
}

fn parentheses(i: &str) -> IResult<&str, SpdxExpression, VerboseError<&str>> {
    delimited(
        char('('),
        alt((
            parentheses,
            and_expression,
            or_expression,
            with_expression,
            simple_expression,
        )),
        char(')'),
    )(i)
}

fn or_expression(i: &str) -> IResult<&str, SpdxExpression, VerboseError<&str>> {
    map(
        tuple((
            alt((ws(parentheses), ws(simple_expression))),
            ws(or),
            alt((
                ws(parentheses),
                ws(and_expression),
                ws(or_expression),
                ws(simple_expression),
            )),
        )),
        |(left, _, right)| SpdxExpression::or(left, right),
    )(i)
}
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, VerboseError<&str>>
where
    F: Fn(&'a str) -> IResult<&'a str, O, VerboseError<&str>>,
{
    delimited(multispace0, inner, multispace0)
}
fn with(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt((tag("WITH"), tag("with")))(i)
}

fn and(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt((tag("AND"), tag("and")))(i)
}

fn or(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    alt((tag("OR"), tag("or")))(i)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        expression::{SimpleExpression, SpdxExpression},
        parse::{
            and_expression, document_ref, idstring, license_idstring, license_ref, or_expression,
            simple_expression, with_expression,
        },
    };

    #[test]
    fn idstring_simple() {
        let (_, result) = idstring("MIT").unwrap();
        assert_eq!(result, "MIT");
    }

    #[test]
    fn idstring_with_numbers() {
        let (_, result) = idstring("GPL-2.0").unwrap();
        assert_eq!(result, "GPL-2.0");
    }

    #[test]
    fn idstring_with_plus() {
        let (_, result) = idstring("GPL-2.0+").unwrap();
        assert_eq!(result, "GPL-2.0");
    }

    #[test]
    fn document_ref_simple() {
        let (_, result) = document_ref("DocumentRef-SPDX-DOC:").unwrap();
        assert_eq!(result, "SPDX-DOC");
    }

    #[test]
    fn license_ref_simple() {
        let (_, result) = license_ref("LicenseRef-Unknown").unwrap();
        assert_eq!(result, (None, "Unknown"));
    }

    #[test]
    fn license_ref_with_document_ref() {
        let (_, result) = license_ref("DocumentRef-Spdx-Doc:LicenseRef-Unknown").unwrap();
        assert_eq!(result, (Some("Spdx-Doc"), "Unknown"));
    }

    #[test]
    fn simple_expression_simple() {
        let (_, result) = simple_expression("MIT").unwrap();
        assert_eq!(
            result,
            SpdxExpression::simple("MIT".to_string(), None, false)
        );
    }

    #[test]
    fn simple_expression_with_licenseref() {
        let (_, result) = simple_expression("LicenseRef-Unknown-license").unwrap();
        assert_eq!(
            result,
            SpdxExpression::simple("Unknown-license".to_string(), None, true)
        );
    }

    #[test]
    fn simple_expression_with_plus() {
        let (_, result) = simple_expression("GPL-2.0+").unwrap();
        assert_eq!(
            result,
            SpdxExpression::simple("GPL-2.0+".to_string(), None, false)
        );
    }

    #[test]
    fn simple_expression_with_document_ref() {
        let (_, result) =
            simple_expression("DocumentRef-SPDX-DOC:LicenseRef-New-license-1.0").unwrap();
        assert_eq!(
            result,
            SpdxExpression::simple(
                "New-license-1.0".to_string(),
                Some("SPDX-DOC".to_string()),
                true
            )
        );
    }

    #[test]
    fn license_idstring_without_plus() {
        let (_, result) = license_idstring("GPL-2.0").unwrap();
        assert_eq!(result, "GPL-2.0");
    }

    #[test]
    fn license_idstring_with_plus() {
        let (_, result) = license_idstring("GPL-2.0+").unwrap();
        assert_eq!(result, "GPL-2.0+");
    }

    #[test]
    fn with_expression_simple() {
        let (_, result) = with_expression("GPL-2.0 WITH Autoconf-exception-2.0").unwrap();
        assert_eq!(
            result,
            SpdxExpression::with(
                SimpleExpression::new("GPL-2.0".to_string(), None, false),
                "Autoconf-exception-2.0".to_string()
            )
        );
    }

    #[test]
    fn and_expression_simple() {
        let (_, result) = and_expression("GPL-2.0 AND MIT").unwrap();
        assert_eq!(
            result,
            SpdxExpression::and(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::simple("MIT".to_string(), None, false),
            )
        );
    }

    #[test]
    fn and_expression_nested() {
        let (_, result) = and_expression("GPL-2.0 AND MIT AND LGPL-2.1").unwrap();
        assert_eq!(
            result,
            SpdxExpression::and(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::and(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false)
                )
            )
        );
    }

    #[test]
    fn and_expression_nested_or() {
        let (_, result) = and_expression("GPL-2.0 AND MIT OR LGPL-2.1").unwrap();
        assert_eq!(
            result,
            SpdxExpression::and(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::or(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false)
                )
            )
        );
    }

    #[test]
    fn and_expression_nested_with() {
        let (_, result) = and_expression("GPL-2.0 AND LGPL-2.1 WITH GCC-exception-2.0").unwrap();
        assert_eq!(
            result,
            SpdxExpression::and(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::with(
                    SimpleExpression::new("LGPL-2.1".to_string(), None, false),
                    "GCC-exception-2.0".to_string()
                )
            )
        );
    }

    #[test]
    fn and_expression_parentheses() {
        let (_, result) = and_expression("(GPL-2.0 AND LGPL-2.1) AND (MIT AND ISC)").unwrap();
        assert_eq!(
            result,
            SpdxExpression::and(
                SpdxExpression::and(
                    SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false),
                ),
                SpdxExpression::and(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("ISC".to_string(), None, false),
                )
            )
        );
    }

    #[test]
    fn and_expression_parentheses_or() {
        let (_, result) = and_expression("(GPL-2.0 AND LGPL-2.1) AND (MIT OR ISC)").unwrap();
        assert_eq!(
            result,
            SpdxExpression::and(
                SpdxExpression::and(
                    SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false),
                ),
                SpdxExpression::or(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("ISC".to_string(), None, false),
                )
            )
        );
    }

    #[test]
    fn or_expression_simple() {
        let (_, result) = or_expression("GPL-2.0 OR MIT").unwrap();
        assert_eq!(
            result,
            SpdxExpression::or(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::simple("MIT".to_string(), None, false)
            )
        );
    }

    #[test]
    fn or_expression_nested() {
        let (_, result) = or_expression("GPL-2.0 OR MIT OR LGPL-2.1").unwrap();
        assert_eq!(
            result,
            SpdxExpression::or(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::or(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false)
                )
            )
        );
    }

    #[test]
    fn or_expression_nested_and() {
        let (_, result) = or_expression("GPL-2.0 OR MIT AND LGPL-2.1").unwrap();
        assert_eq!(
            result,
            SpdxExpression::or(
                SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                SpdxExpression::and(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false)
                )
            )
        );
    }

    #[test]
    fn or_expression_parentheses() {
        let (_, result) = or_expression("(GPL-2.0 AND LGPL-2.1) OR (MIT OR ISC)").unwrap();
        assert_eq!(
            result,
            SpdxExpression::or(
                SpdxExpression::and(
                    SpdxExpression::simple("GPL-2.0".to_string(), None, false),
                    SpdxExpression::simple("LGPL-2.1".to_string(), None, false),
                ),
                SpdxExpression::or(
                    SpdxExpression::simple("MIT".to_string(), None, false),
                    SpdxExpression::simple("ISC".to_string(), None, false),
                )
            )
        );
    }
}
