// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

mod error;
mod expression;
mod inner_variant;
mod parser;

pub use error::SpdxExpressionError;
pub use expression::SPDXExpression;
