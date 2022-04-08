// SPDX-FileCopyrightText: 2022 HH Partners
//
// SPDX-License-Identifier: MIT

//! # SPDX License Expressions in Rust
//!
//! Library for parsing and interacting with [SPDX License Expressions][spdx-expression]. The main
//! functionality is accessed through the [`SPDXExpression`] struct.
//!
//! [spdx-expression]: https://spdx.github.io/spdx-spec/SPDX-license-expressions/

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

mod error;
mod expression;
mod inner_variant;
mod parser;

pub use error::SpdxExpressionError;
pub use expression::SPDXExpression;
