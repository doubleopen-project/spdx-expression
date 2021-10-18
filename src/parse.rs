use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::streaming::char,
    combinator::{complete, map, opt},
    error::VerboseError,
    sequence::{delimited, separated_pair, terminated},
    AsChar, IResult,
};

use crate::expression::SimpleExpression;

fn idstring(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(|c: char| c.is_alphanum() || c == '-' || c == '.')(i)
}

fn document_ref(i: &str) -> IResult<&str, &str, VerboseError<&str>> {
    delimited(tag("DocumentRef-"), idstring, char(':'))(i)
}

fn license_ref(i: &str) -> IResult<&str, (Option<&str>, &str), VerboseError<&str>> {
    separated_pair(opt(document_ref), tag("LicenseRef-"), idstring)(i)
}

fn simple_expression(i: &str) -> IResult<&str, SimpleExpression, VerboseError<&str>> {
    alt((
        map(license_ref, |(document_ref, id)| {
            let document_ref = document_ref.map(|document_ref| document_ref.to_string());
            SimpleExpression::new(id.to_string(), document_ref, true)
        }),
        map(complete(terminated(idstring, char('+'))), |id| {
            SimpleExpression::new(format!("{}+", id), None, false)
        }),
        map(idstring, |id| {
            SimpleExpression::new(id.to_string(), None, false)
        }),
    ))(i)
}
#[cfg(test)]
mod tests {
    use crate::{
        expression::SimpleExpression,
        parse::{document_ref, idstring, license_ref, simple_expression},
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
            SimpleExpression {
                document_ref: None,
                identifier: "MIT".to_string(),
                license_ref: false
            }
        );
    }

    #[test]
    fn simple_expression_with_licenseref() {
        let (_, result) = simple_expression("LicenseRef-Unknown-license").unwrap();
        assert_eq!(
            result,
            SimpleExpression {
                document_ref: None,
                identifier: "Unknown-license".to_string(),
                license_ref: true
            }
        );
    }

    #[test]
    fn simple_expression_with_plus() {
        let (_, result) = simple_expression("GPL-2.0+").unwrap();
        assert_eq!(
            result,
            SimpleExpression {
                document_ref: None,
                identifier: "GPL-2.0+".to_string(),
                license_ref: false
            }
        );
    }

    #[test]
    fn simple_expression_with_document_ref() {
        let (_, result) =
            simple_expression("DocumentRef-SPDX-DOC:LicenseRef-New-license-1.0").unwrap();
        assert_eq!(
            result,
            SimpleExpression {
                document_ref: Some("SPDX-DOC".to_string()),
                identifier: "New-license-1.0".to_string(),
                license_ref: true
            }
        );
    }
}
