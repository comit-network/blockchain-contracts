use crate::calculate_offsets::check_bin_in_path;
use anyhow::Context;
use anyhow::Result;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub fn compile<S: AsRef<Path>>(file_path: S) -> Result<Vec<u8>> {
    check_bin_in_path("docker");
    let mut bx = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-i")
        .arg("coblox/libbitcoin-explorer:latest")
        .arg("script-encode")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let file_path = file_path.as_ref();
    let input = std::fs::read(file_path)
        .with_context(|| format!("failed to open contract file {}", file_path.display()))?;
    let input = String::from_utf8(input)?;
    let input = input.replace("\n", " ").into_bytes();

    let stdin = bx.stdin.as_mut().context("failed to write to stdin")?;
    stdin.write_all(&input)?;

    let output = bx.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let bytes = hex::decode(stdout.trim())?;

    Ok(bytes)
}
