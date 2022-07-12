use anyhow::{bail, Result};
use std::process::Command;

/// Format a `Command` as a `String.
///
/// Example: "VAR=val program --arg1 arg2".
pub fn command_to_string(cmd: &Command) -> String {
    // Format env vars as "name=val".
    let ignore_var = ["PATH", "RUSTC", "RUSTDOC"];
    let mut parts = cmd
        .get_envs()
        // Filter out some internally-set variables that would just
        // clutter the output.
        .filter(|(name, _)| !ignore_var.contains(&name.to_str().unwrap_or_default()))
        .map(|(name, val)| {
            format!(
                "{}={}",
                name.to_string_lossy(),
                val.unwrap_or_default().to_string_lossy()
            )
        })
        .collect::<Vec<_>>();

    // Add the program name.
    parts.push(cmd.get_program().to_string_lossy().to_string());

    // Add each argument.
    parts.extend(cmd.get_args().map(|arg| arg.to_string_lossy().to_string()));

    // Join the vars, program, and arguments into a single string.
    parts.into_iter().collect::<Vec<_>>().join(" ")
}

/// Print a `Command` and run it, then check that it completes
/// successfully.
pub fn run_cmd(mut cmd: Command) -> Result<()> {
    println!("{}", command_to_string(&cmd));

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        bail!("command failed: {}", status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_to_string() {
        let mut cmd = Command::new("MyCommand");
        cmd.args(&["abc", "123"]).envs([
            ("VAR1", "val1"),
            ("VAR2", "val2"),
            ("PATH", "pathval"),
            ("RUSTC", "rustcval"),
            ("RUSTDOC", "rustdocval"),
        ]);
        assert_eq!(
            command_to_string(&cmd),
            "VAR1=val1 VAR2=val2 MyCommand abc 123"
        );
    }
}
