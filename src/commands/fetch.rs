// This is free and unencumbered software released into the public domain.

use crate::{StandardOptions, SysexitsError};
use color_print::ceprintln;
use miette::{IntoDiagnostic, Result};
use std::io::stdout;

pub fn fetch(urls: &Vec<String>, flags: &StandardOptions) -> Result<(), SysexitsError> {
    let mut output = stdout().lock();

    for url in urls {
        if flags.verbose > 1 {
            if url.contains(':') {
                ceprintln!("<s><c>»</></> Fetching `{}`...", url);
            } else {
                ceprintln!("<s><c>»</></> Reading `{}`...", url);
            }
        }

        let mut input = match asimov_proxy::open(url) {
            Ok(mut input) => input,
            Err(error) => {
                let code = SysexitsError::from(&error);
                let report: miette::Report = FetchError {
                    url: url.clone(),
                    cause: error,
                }
                .into();
                eprintln!("{:?}", report);
                return Err(code);
            }
        };

        std::io::copy(&mut input, &mut output)?;

        if flags.verbose > 0 {
            if url.contains(':') {
                ceprintln!("<s><g>✓</></> Fetched `{}`.", url); // TODO: time duration
            } else {
                ceprintln!("<s><g>✓</></> Read `{}`.", url);
            }
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("Failed to fetch URL: {url}")]
#[diagnostic(code("× Error"), url(docsrs))]
struct FetchError {
    url: String,

    #[source]
    #[diagnostic_source]
    cause: asimov_proxy::Error,
}
