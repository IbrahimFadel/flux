use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum PIErrorCode {
    LexInvalidDigit,
}

#[derive(Debug, Clone, Copy)]
pub struct PIError {
    msg: &'static str,
    code: PIErrorCode,
}
