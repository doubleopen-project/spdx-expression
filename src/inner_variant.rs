// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

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
pub struct WithExpression {
    pub license: SimpleExpression,
    pub exception: String,
}

impl WithExpression {
    pub const fn new(license: SimpleExpression, exception: String) -> Self {
        Self { license, exception }
    }
}
