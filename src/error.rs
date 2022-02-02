#[derive(thiserror::Error, Debug)]
pub enum SpdxExpressionError {
    #[error("Parsing for expression `{0}` failed.")]
    Parse(String),
}
