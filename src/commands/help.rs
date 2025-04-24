// This is free and unencumbered software released into the public domain.

use clientele::SubcommandsProvider;
use clientele::SysexitsError::{self, *};
use rayon::prelude::*;
use std::process::Stdio;

use crate::Result;

pub struct CommandDescription {
    pub name: String,
    pub description: String,
    pub usage: Option<String>,
}

/// Prints extensive help message, executing `help` command for each subcommand.
pub struct Help {
    pub is_debug: bool,
}

impl Help {
    pub fn execute(&self) -> Vec<CommandDescription> {
        let output = self.collect_output();

        let mut result = vec![];
        for (name, description) in output {
            let lines = description.lines().collect::<Vec<_>>();
            let description = lines[0];
            let usage = lines.iter().find(|line| line.starts_with("Usage:"));

            result.push(CommandDescription {
                name,
                description: description.to_string(),
                usage: usage.map(|usage| usage.to_string()),
            });
        }

        result
    }

    fn is_child_running(&self, child: &mut std::process::Child) -> Result<bool, std::io::Error> {
        Ok(child.try_wait()?.is_none())
    }

    fn collect_output(&self) -> Vec<(String, String)> {
        const MAX_WAIT_TIME: std::time::Duration = std::time::Duration::from_secs(1);

        let commands = SubcommandsProvider::collect("asimov-", 1);

        // FIXME: DIRTY DIRTY HACK!
        // We need to clone the subcommands to obtain Vec<_> from SubcommandsProvider,
        // since it doesn't implement into_iter() or into_vec().
        // Fix it with a new release of `clientele`.
        let commands = commands.iter().cloned().collect::<Vec<_>>();

        let start_time = std::time::Instant::now();

        commands
            .par_iter()
            .filter_map(|cmd| {
                let Ok(mut child) = std::process::Command::new(&cmd.path)
                    .args(["--help"])
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                else {
                    return None;
                };

                loop {
                    let Ok(is_running) = self.is_child_running(&mut child) else {
                        return None;
                    };

                    if !is_running {
                        break;
                    }

                    let now = std::time::Instant::now();
                    if now.duration_since(start_time) > MAX_WAIT_TIME {
                        child.kill();
                        drop(child);
                        return None;
                    }

                    rayon::yield_local();
                }

                let output = child.wait_with_output().unwrap();
                if !output.status.success() {
                    return None;
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                Some((cmd.name.clone(), stdout.to_string()))
            })
            .collect()
    }
}
