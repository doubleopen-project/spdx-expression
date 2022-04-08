// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

//! Errors of the library.

/// Custom error struct.
#[derive(thiserror::Error, Debug)]
pub enum SpdxExpressionError {
    #[error("Parsing for expression `{0}` failed.")]
    Parse(String),
}
