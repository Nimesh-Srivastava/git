use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::io::BufReader;
use std::io::prelude::*;

pub(crate) fn invoke(name_only: bool) -> anyhow::Result<()> {
    anyhow::ensure!(
        name_only,
        "--name_only is the only subcommand supported for now"
    );
    Ok(())
}
