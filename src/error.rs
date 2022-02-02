// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

#[derive(thiserror::Error, Debug)]
pub enum SpdxExpressionError {
    #[error("Parsing for expression `{0}` failed.")]
    Parse(String),
}
