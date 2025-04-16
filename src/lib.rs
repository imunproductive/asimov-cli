// This is free and unencumbered software released into the public domain.

pub mod commands;
pub mod shared;

use clientele::SysexitsError;

pub type Result<T = SysexitsError, E = SysexitsError> = std::result::Result<T, E>;
