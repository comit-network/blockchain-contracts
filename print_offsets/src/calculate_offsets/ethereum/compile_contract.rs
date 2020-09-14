use crate::calculate_offsets::check_bin_in_path;
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use std::{
    env::var,
    process::{Command, Stdio},
};

pub fn compile<S: AsRef<Path>>(file_path: S) -> Result<Vec<u8>> {
    let solc_bin = var("SOLC_BIN");

    let mut solc = match solc_bin {
        Ok(bin) => Command::new(bin)
            .arg("--assemble")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?,
        Err(_) => {
            check_bin_in_path("docker");
            Command::new("docker")
                .arg("run")
                .arg("--rm")
                .arg("-i")
                .arg("ethereum/solc:0.5.16")
                .arg("--assemble")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?
        }
    };

    let file_path = file_path.as_ref();
    let mut file = ::std::fs::File::open(file_path)
        .with_context(|| format!("failed to open contract file {}", file_path.display()))?;

    ::std::io::copy(&mut file, solc.stdin.as_mut().unwrap())?;

    let output = solc.wait_with_output()?;
    let stdout = String::from_utf8(output.stdout).unwrap();
    let regex = Regex::new(r"\nBinary representation:\n(?P<hexcode>.+)\n")?;

    let captures = regex
        .captures(stdout.as_str())
        .expect("Regex didn't match!");

    let hexcode = captures
        .name("hexcode")
        .context("failed to find hex in solc output")?;
    let bytes = hex::decode(hexcode.as_str())?;

    Ok(bytes)
}
