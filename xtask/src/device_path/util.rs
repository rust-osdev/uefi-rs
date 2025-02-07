// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use syn::Attribute;

/// Returns true if the attribute is a `#[doc = "..."]` attribute,
/// otherwise returns false.
pub fn is_doc_attr(attr: &Attribute) -> bool {
    attr.path().is_ident("doc")
}

/// Run `rustfmt` on the `input` string and return the formatted code.
pub fn rustfmt_string(input: String) -> Result<String> {
    let mut child = Command::new("rustfmt")
        .args([
            "--config",
            // Convert `#[doc = "..."]` to `///` for readability.
            "normalize_doc_attributes=true",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write on a separate thread to avoid deadlock.
    let mut stdin = child.stdin.take().context("failed to take stdin")?;
    thread::spawn(move || {
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    });

    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("rustfmt failed");
    }

    let stdout = String::from_utf8(output.stdout)?;

    Ok(stdout)
}
