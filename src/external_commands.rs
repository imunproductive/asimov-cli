// This is free and unencumbered software released into the public domain.

use std::{fs::ReadDir, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct ExternalCommand {
    pub name: String,
    pub path: PathBuf,
}

pub struct ExternalCommands {
    commands: Vec<ExternalCommand>,
}

impl ExternalCommands {
    pub fn collect(prefix: &str, level: usize) -> Result<ExternalCommands> {
        let commands = Self::collect_internal(prefix, level)?;
        Ok(ExternalCommands { commands })
    }

    fn collect_internal(prefix: &str, level: usize) -> Result<Vec<ExternalCommand>> {
        let Some(paths) = std::env::var_os("PATH") else {
            // PATH variable is not set.
            return Ok(vec![]);
        };

        let mut result = vec![];
        for path in std::env::split_paths(&paths) {
            let Ok(dir) = std::fs::read_dir(path) else {
                continue;
            };

            let commands = Self::collect_in_dir(prefix, dir)?;
            result.extend(commands.into_iter().filter(|cmd| {
                let count = cmd.name.chars().filter(|&c| c == '-').count();
                count < level
            }));
        }

        Ok(result)
    }
}

impl ExternalCommands {
    pub fn iter(&self) -> impl Iterator<Item = &ExternalCommand> {
        self.commands.iter()
    }

    pub fn find(&self, name: &str) -> Option<&ExternalCommand> {
        self.commands.iter().find(|command| command.name == name)
    }
}

#[cfg(unix)]
impl ExternalCommands {
    // use std::fs::Metadata;

    fn is_executable_path(path: impl AsRef<Path>) -> bool {
        std::fs::metadata(path)
            .map(is_executable_metadata)
            .unwrap_or(false)
    }

    fn is_executable_metadata(metadata: Metadata) -> bool {
        use std::os::unix::prelude::*;
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }

    fn collect_in_dir(prefix: &[u8], dir: ReadDir) -> Result<Vec<ExternalCommand>> {
        use std::os::unix::ffi::OsStrExt;

        for entry in dir {
            let entry = entry?;
            let entry_name = entry.file_name();
            let entry_bytes = entry_name.as_bytes();
            if entry_bytes.starts_with(b".")
                || entry_bytes.ends_with(b"~")
                || !entry_bytes.starts_with(prefix)
                || !is_executable_metadata(entry.metadata()?)
            {
                continue; // skip hidden, backup, and non-executable files
            }
            let count = entry_bytes
                .iter()
                .fold(0, |sum, &b| sum + (b == b'-') as usize);
            if count <= level {
                result.push(entry.path());
            }
        }
    }
}

#[cfg(windows)]
impl ExternalCommands {
    fn collect_in_dir(prefix: &str, dir: ReadDir) -> Result<Vec<ExternalCommand>> {
        use std::env::consts::EXE_EXTENSION;
        use std::os::windows::fs::MetadataExt;
        use std::path::PathBuf;

        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;

        // FIXME: Currently we don't handle the case when
        // there are multiple files with the same name,
        // but different extensions. This is a bug.
        static BAT_EXTENSION: &str = "bat";
        static CMD_EXTENSION: &str = "cmd";
        static ALLOWED_EXTENSIONS: [&str; 3] = [BAT_EXTENSION, CMD_EXTENSION, EXE_EXTENSION];

        let mut result = vec![];

        for entry in dir {
            let Ok(entry) = entry else {
                // invalid entry.
                continue;
            };

            let file_name = entry.file_name();
            let Some(entry_name) = file_name.to_str() else {
                // skip files with invalid names.
                continue;
            };

            if !entry_name.starts_with(prefix) {
                // skip non-matching files.
                continue;
            }

            let entry_path = entry.path();
            let Some(entry_ext) = entry_path.extension().and_then(|ext| ext.to_str()) else {
                // skip files without extensions.
                continue;
            };

            let entry_ext = entry_ext.to_lowercase();
            if !ALLOWED_EXTENSIONS.contains(&entry_ext.as_str()) {
                // skip non-executable files
                continue;
            }

            let metadata = entry.metadata()?;
            let is_hidden = metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
            if is_hidden {
                // skip hidden files
                continue;
            }

            let name_path = PathBuf::from(entry_name);
            let Some(name) = name_path.file_stem().map(|name| {
                name.to_string_lossy()
                    .trim_start_matches(prefix)
                    .to_string()
            }) else {
                continue;
            };

            result.push(ExternalCommand {
                name,
                path: entry_path,
            });
        }

        Ok(result)
    }
}
