// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError};

pub fn fetch(_urls: &Vec<String>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    Err(SysexitsError::EX_UNAVAILABLE) // TODO
}
