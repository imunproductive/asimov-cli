// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError};
use std::io::stdout;

pub fn fetch(urls: &Vec<String>, _flags: &StandardOptions) -> Result<(), SysexitsError> {
    let mut output = stdout().lock();

    for url in urls {
        match asimov_proxy::open(url) {
            Ok(mut input) => {
                std::io::copy(&mut input, &mut output)?;
            }
            Err(error) => {
                eprintln!("failed to fetch URL `{}`: {}", url, error);
                return Err(error.into());
            }
        }
    }

    Ok(())
}
